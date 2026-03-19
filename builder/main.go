package main

import (
	"bytes"
	"fmt"
	"io"
	"os"
	"os/exec"
	"path/filepath"
	"sort"
	"strings"

	"github.com/BurntSushi/toml"
	"github.com/charmbracelet/bubbles/spinner"
	"github.com/charmbracelet/bubbles/textinput"
	"github.com/charmbracelet/bubbles/viewport"
	tea "github.com/charmbracelet/bubbletea"
	"github.com/charmbracelet/lipgloss"
)

type cargoManifest struct {
	Features map[string][]string `toml:"features"`
	Bins     []cargoBin          `toml:"bin"`
}

type cargoBin struct {
	Name string `toml:"name"`
	Path string `toml:"path"`
}

type option struct {
	Label    string
	Value    string
	Selected bool
}

type step int

const (
	stepSearch step = iota
	stepPruning
	stepOrdering
	stepEval
	stepBuild
)

type buildResult struct {
	Err       error
	Command   string
	Output    string
	BuiltPath string
}

type buildResultMsg struct {
	result buildResult
}

type model struct {
	repoRoot string

	search   []option
	pruning  []option
	ordering []option
	evals    []option

	step   step
	cursor int

	modeRelease  bool
	debugSymbols bool

	outputInput textinput.Model
	spinner     spinner.Model
	viewport    viewport.Model

	width  int
	height int

	building  bool
	buildLog  string
	lastCmd   string
	lastError string
	builtPath string
}

var (
	bg             = lipgloss.Color("#0A0F1C")
	panel          = lipgloss.Color("#101726")
	panelAlt       = lipgloss.Color("#0D1422")
	border         = lipgloss.Color("#24324A")
	text           = lipgloss.Color("#E8EEF8")
	muted          = lipgloss.Color("#8191A9")
	accent         = lipgloss.Color("#56D4DD")
	accentDim      = lipgloss.Color("#153A43")
	success        = lipgloss.Color("#34D399")
	errorColor     = lipgloss.Color("#F87171")
	warn           = lipgloss.Color("#FBBF24")
	titleStyle     = lipgloss.NewStyle().Foreground(text).Bold(true)
	mutedStyle     = lipgloss.NewStyle().Foreground(muted)
	headerStyle    = lipgloss.NewStyle().Foreground(accent).Bold(true)
	focusedStyle   = lipgloss.NewStyle().Foreground(text).Background(accentDim).Bold(true).Padding(0, 1)
	selectedStyle  = lipgloss.NewStyle().Foreground(accent).Bold(true)
	errorStyle     = lipgloss.NewStyle().Foreground(errorColor).Bold(true)
	successStyle   = lipgloss.NewStyle().Foreground(success).Bold(true)
	chipStyle      = lipgloss.NewStyle().Foreground(text).Background(panelAlt).Border(lipgloss.RoundedBorder()).BorderForeground(border).Padding(0, 1)
	chipFocusStyle = lipgloss.NewStyle().Foreground(bg).Background(accent).Border(lipgloss.RoundedBorder()).BorderForeground(accent).Bold(true).Padding(0, 1)
	frameStyle     = lipgloss.NewStyle().Background(bg).Padding(1, 2)
	cardStyle      = lipgloss.NewStyle().Background(panel).Border(lipgloss.RoundedBorder()).BorderForeground(border).Padding(1, 2)
)

func main() {
	repoRoot, err := findRepoRoot()
	if err != nil {
		fmt.Fprintf(os.Stderr, "builder: %v\n", err)
		os.Exit(1)
	}

	manifest, err := loadCargoManifest(filepath.Join(repoRoot, "Cargo.toml"))
	if err != nil {
		fmt.Fprintf(os.Stderr, "builder: %v\n", err)
		os.Exit(1)
	}

	m := newModel(repoRoot, manifest)
	_, err = tea.NewProgram(m, tea.WithAltScreen()).Run()
	if err != nil {
		fmt.Fprintf(os.Stderr, "builder: %v\n", err)
		os.Exit(1)
	}
}

func newModel(repoRoot string, manifest cargoManifest) model {
	input := textinput.New()
	input.Prompt = ""
	input.SetValue("oopsmate-custom")
	input.CharLimit = 64
	input.Width = 28

	spin := spinner.New()
	spin.Spinner = spinner.Dot
	spin.Style = lipgloss.NewStyle().Foreground(accent)

	evals := discoverEvalOptions(manifest.Bins)
	if len(evals) > 0 {
		evals[0].Selected = true
	}

	return model{
		repoRoot:     repoRoot,
		search:       discoverFeatureOptions(manifest.Features, []string{"pvs", "aspiration-windows", "iid", "singular-extensions", "check-extensions"}),
		pruning:      discoverFeatureOptions(manifest.Features, []string{"null-move", "lmr", "futility", "reverse-futility", "razoring", "probcut"}),
		ordering:     discoverFeatureOptions(manifest.Features, []string{"tt-move-ordering", "killer-moves", "history-heuristic", "see"}),
		evals:        evals,
		step:         stepSearch,
		modeRelease:  true,
		debugSymbols: true,
		outputInput:  input,
		spinner:      spin,
	}
}

func (m model) Init() tea.Cmd {
	return textinput.Blink
}

func (m model) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
	if m.building {
		var cmds []tea.Cmd
		var cmd tea.Cmd
		m.spinner, cmd = m.spinner.Update(msg)
		cmds = append(cmds, cmd)

		switch msg := msg.(type) {
		case tea.KeyMsg:
			if msg.String() == "ctrl+c" {
				return m, tea.Quit
			}
		case buildResultMsg:
			m.building = false
			m.lastCmd = msg.result.Command
			m.buildLog = strings.TrimSpace(msg.result.Output)
			if msg.result.Err != nil {
				m.lastError = msg.result.Err.Error()
			} else {
				m.lastError = ""
				m.builtPath = msg.result.BuiltPath
			}
			m.refreshViewport()
		}
		return m, tea.Batch(cmds...)
	}

	switch msg := msg.(type) {
	case tea.WindowSizeMsg:
		m.width = msg.Width
		m.height = msg.Height
		m.outputInput.Width = max(22, min(42, msg.Width/3))
		m.refreshViewport()
		return m, nil
	case tea.KeyMsg:
		switch msg.String() {
		case "ctrl+c", "q":
			return m, tea.Quit
		case "h":
			m.prevStep()
		case "l":
			m.nextStep()
		case "tab":
			m.nextStep()
		case "shift+tab":
			m.prevStep()
		case "k":
			m.moveCursor(-1)
		case "j":
			m.moveCursor(1)
		case " ", "enter":
			return m.activate(msg.String() == "enter")
		default:
			if m.step == stepBuild {
				var cmd tea.Cmd
				m.outputInput, cmd = m.outputInput.Update(msg)
				m.refreshViewport()
				return m, cmd
			}
		}
	}

	m.refreshViewport()
	return m, nil
}

func (m model) View() string {
	if m.width == 0 {
		return "loading builder..."
	}

	hero := lipgloss.JoinVertical(
		lipgloss.Left,
		titleStyle.Render("Oopsmate Builder"),
	)

	content := lipgloss.JoinVertical(
		lipgloss.Left,
		m.renderStepper(),
		m.renderStageCard(),
		m.renderFooter(),
	)

	return frameStyle.Render(lipgloss.JoinVertical(lipgloss.Left, hero, "", content))
}

func (m *model) activate(fromEnter bool) (tea.Model, tea.Cmd) {
	switch m.step {
	case stepSearch:
		if m.cursor == len(m.search) {
			m.toggleAll(&m.search)
		} else if len(m.search) > 0 {
			m.search[m.cursor].Selected = !m.search[m.cursor].Selected
		}
	case stepPruning:
		if m.cursor == len(m.pruning) {
			m.toggleAll(&m.pruning)
		} else if len(m.pruning) > 0 {
			m.pruning[m.cursor].Selected = !m.pruning[m.cursor].Selected
		}
	case stepOrdering:
		if m.cursor == len(m.ordering) {
			m.toggleAll(&m.ordering)
		} else if len(m.ordering) > 0 {
			m.ordering[m.cursor].Selected = !m.ordering[m.cursor].Selected
		}
	case stepEval:
		for i := range m.evals {
			m.evals[i].Selected = i == m.cursor
		}
		if fromEnter {
			m.nextStep()
		}
	case stepBuild:
		switch m.cursor {
		case 0:
			if fromEnter {
				m.outputInput.Focus()
			}
		case 1:
			m.modeRelease = !m.modeRelease
		case 2:
			if m.modeRelease {
				m.debugSymbols = !m.debugSymbols
			}
		case 3:
			cmd, err := m.prepareBuild()
			if err != nil {
				m.lastError = err.Error()
				m.buildLog = ""
				m.refreshViewport()
				return m, nil
			}
			m.building = true
			m.lastError = ""
			m.buildLog = ""
			m.builtPath = ""
			m.refreshViewport()
			return m, tea.Batch(m.spinner.Tick, cmd)
		}
	}

	m.refreshViewport()
	return m, nil
}

func (m *model) moveCursor(delta int) {
	count := m.currentOptionCount()
	if count == 0 {
		m.cursor = 0
		return
	}
	m.cursor = (m.cursor + delta + count) % count
	if m.step == stepBuild && m.cursor == 0 {
		m.outputInput.Focus()
	} else {
		m.outputInput.Blur()
	}
}

func (m *model) nextStep() {
	if m.step < stepBuild {
		m.step++
		m.cursor = 0
	}
	m.syncFocus()
}

func (m *model) prevStep() {
	if m.step > stepSearch {
		m.step--
		m.cursor = 0
	}
	m.syncFocus()
}

func (m *model) syncFocus() {
	count := m.currentOptionCount()
	if count == 0 {
		m.cursor = 0
	} else if m.cursor >= count {
		m.cursor = count - 1
	}
	if m.step == stepBuild && m.cursor == 0 {
		m.outputInput.Focus()
	} else {
		m.outputInput.Blur()
	}
	m.refreshViewport()
}

func (m model) currentOptionCount() int {
	switch m.step {
	case stepSearch:
		return len(m.search) + 1
	case stepPruning:
		return len(m.pruning) + 1
	case stepOrdering:
		return len(m.ordering) + 1
	case stepEval:
		return len(m.evals)
	case stepBuild:
		if m.modeRelease {
			return 4
		}
		return 3
	default:
		return 0
	}
}

func (m model) renderStepper() string {
	steps := []string{"Search", "Pruning", "Ordering", "Eval", "Build"}
	chips := make([]string, 0, len(steps))
	for i, name := range steps {
		style := chipStyle
		if int(m.step) == i {
			style = chipFocusStyle
		}
		chips = append(chips, style.Render(fmt.Sprintf("%d %s", i+1, name)))
	}
	return lipgloss.JoinHorizontal(lipgloss.Top, chips...)
}

func (m model) renderStageCard() string {
	var body string
	switch m.step {
	case stepSearch:
		body = m.renderOptionStep("Search", "Baseline: alpha-beta", m.search, false)
	case stepPruning:
		body = m.renderOptionStep("Pruning", "Forward pruning and reductions. Keep this lean when you want isolated Elo numbers.", m.pruning, false)
	case stepOrdering:
		body = m.renderOptionStep("Ordering", "Move ordering and capture scoring heuristics.", m.ordering, false)
	case stepEval:
		body = m.renderOptionStep("Eval Provider", "Pick the binary backend that will be built.", m.evals, true)
	case stepBuild:
		body = m.renderBuildStep()
	}
	return cardStyle.Width(max(72, m.width-8)).Render(body)
}

func (m model) renderOptionStep(title, subtitle string, items []option, single bool) string {
	rows := []string{headerStyle.Render(title), mutedStyle.Render(subtitle), ""}
	for i, item := range items {
		marker := "[ ]"
		if single {
			marker = "( )"
		}
		if item.Selected {
			if single {
				marker = "(•)"
			} else {
				marker = "[x]"
			}
		}
		line := fmt.Sprintf("%s %s", marker, item.Label)
		if i == m.cursor {
			rows = append(rows, focusedStyle.Render(line))
		} else if item.Selected {
			rows = append(rows, selectedStyle.Render(line))
		} else {
			rows = append(rows, line)
		}
	}
	if !single {
		selectAllLabel := fmt.Sprintf("[%s] Select all", ternary(allSelected(items), "x", " "))
		if m.cursor == len(items) {
			rows = append(rows, focusedStyle.Render(selectAllLabel))
		} else {
			rows = append(rows, chipStyle.Render(selectAllLabel))
		}
	}
	rows = append(rows, "")
	if !single && title == "Search" {
		rows = append(rows, mutedStyle.Render("Baseline: alpha-beta"))
	}
	rows = append(rows, mutedStyle.Render(fmt.Sprintf("Selected: %s", m.selectedSummary(items, single))))
	rows = append(rows, mutedStyle.Render(m.commandPreview()))
	return strings.Join(rows, "\n")
}

func (m model) renderBuildStep() string {
	rows := []string{
		headerStyle.Render("Build"),
		mutedStyle.Render("Name the artifact, choose debug or release, then compile it."),
		"",
	}

	outputLabel := fmt.Sprintf("Output binary: %s", sanitizeOutputName(m.outputInput.Value()))
	if m.cursor == 0 {
		rows = append(rows, focusedStyle.Render(outputLabel))
		rows = append(rows, m.outputInput.View())
	} else {
		rows = append(rows, outputLabel)
		rows = append(rows, mutedStyle.Render(m.outputInput.View()))
	}

	modeText := fmt.Sprintf("Build mode: %s", ternary(m.modeRelease, "release", "debug"))
	if m.cursor == 1 {
		rows = append(rows, focusedStyle.Render(modeText))
	} else {
		rows = append(rows, modeText)
	}

	if m.modeRelease {
		symbolsText := fmt.Sprintf("Debug symbols: %s", ternary(m.debugSymbols, "on", "off"))
		if m.cursor == 2 {
			rows = append(rows, focusedStyle.Render(symbolsText))
		} else {
			rows = append(rows, symbolsText)
		}
	}

	buildCursor := 2
	if m.modeRelease {
		buildCursor = 3
	}
	buildLabel := "Build"
	if m.building {
		buildLabel = fmt.Sprintf("%s compiling...", m.spinner.View())
	}
	if m.cursor == buildCursor {
		rows = append(rows, focusedStyle.Render(buildLabel))
	} else {
		rows = append(rows, chipStyle.Render(buildLabel))
	}

	rows = append(rows, "")
	rows = append(rows, mutedStyle.Render("Baseline: alpha-beta"))
	rows = append(rows, mutedStyle.Render(fmt.Sprintf("Eval: %s", m.selectedEval().Label)))
	rows = append(rows, mutedStyle.Render(fmt.Sprintf("Features: %s", strings.Join(m.selectedFeatures(), ", "))))
	rows = append(rows, mutedStyle.Render(m.commandPreview()))
	if m.lastCmd != "" {
		rows = append(rows, mutedStyle.Render(m.lastCmd))
	}
	if m.lastError != "" {
		rows = append(rows, errorStyle.Render(m.lastError))
	} else if m.builtPath != "" {
		rows = append(rows, successStyle.Render(fmt.Sprintf("Built at %s", m.builtPath)))
	}

	rows = append(rows, "")
	rows = append(rows, m.viewport.View())
	return strings.Join(rows, "\n")
}

func (m model) renderFooter() string {
	helpLine := "j/k move  space toggle  enter activate  h/l step  q quit"
	return mutedStyle.Render(helpLine)
}

func (m *model) prepareBuild() (tea.Cmd, error) {
	bin := m.selectedEval()
	if bin.Value == "" {
		return nil, fmt.Errorf("select an eval provider first")
	}

	outputName := sanitizeOutputName(strings.TrimSpace(m.outputInput.Value()))
	if outputName == "" {
		return nil, fmt.Errorf("output binary name cannot be empty")
	}

	features := m.selectedFeatures()
	args := []string{"build", "--bin", bin.Value}
	if m.modeRelease {
		args = append(args, "--release")
	}
	if len(features) > 0 {
		args = append(args, "--features", strings.Join(features, ","))
	}

	commandDisplay := "cargo " + strings.Join(args, " ")
	profileDir := "debug"
	if m.modeRelease {
		profileDir = "release"
	}
	sourcePath := filepath.Join(m.repoRoot, "target", profileDir, bin.Value)
	builtPath := filepath.Join(m.repoRoot, "target", profileDir, outputName)

	env := os.Environ()
	if m.modeRelease {
		env = append(env, fmt.Sprintf("CARGO_PROFILE_RELEASE_DEBUG=%t", m.debugSymbols))
	}

	return func() tea.Msg {
		cmd := exec.Command("cargo", args...)
		cmd.Dir = m.repoRoot
		cmd.Env = env
		var combined bytes.Buffer
		cmd.Stdout = &combined
		cmd.Stderr = &combined
		err := cmd.Run()
		if err != nil {
			return buildResultMsg{result: buildResult{Err: err, Command: commandDisplay, Output: combined.String()}}
		}

		if copyErr := copyExecutable(sourcePath, builtPath); copyErr != nil {
			return buildResultMsg{result: buildResult{Err: copyErr, Command: commandDisplay, Output: combined.String()}}
		}

		return buildResultMsg{result: buildResult{Command: commandDisplay, Output: combined.String(), BuiltPath: builtPath}}
	}, nil
}

func (m model) refreshViewport() {
	width := max(50, m.width-18)
	height := max(8, m.height-24)
	if m.viewport.Width == 0 {
		m.viewport = viewport.New(width, height)
	} else {
		m.viewport.Width = width
		m.viewport.Height = height
	}
	content := m.buildLog
	if content == "" {
		content = "Cargo output appears here once you build."
	}
	m.viewport.SetContent(content)
	m.viewport.GotoBottom()
}

func (m model) selectedEval() option {
	for _, item := range m.evals {
		if item.Selected {
			return item
		}
	}
	if len(m.evals) > 0 {
		return m.evals[0]
	}
	return option{}
}

func (m model) selectedFeatures() []string {
	features := make([]string, 0, len(m.search)+len(m.pruning)+len(m.ordering))
	for _, group := range [][]option{m.search, m.pruning, m.ordering} {
		for _, item := range group {
			if item.Selected {
				features = append(features, item.Value)
			}
		}
	}
	sort.Strings(features)
	return features
}

func (m model) selectedSummary(items []option, single bool) string {
	selected := make([]string, 0, len(items))
	for _, item := range items {
		if item.Selected {
			selected = append(selected, item.Label)
		}
	}
	if len(selected) == 0 {
		if single {
			return "none"
		}
		return "none"
	}
	return strings.Join(selected, ", ")
}

func (m model) commandPreview() string {
	bin := m.selectedEval().Value
	if bin == "" {
		bin = "<eval-bin>"
	}
	args := []string{"cargo", "build", "--bin", bin}
	if m.modeRelease {
		args = append(args, "--release")
	}
	features := m.selectedFeatures()
	if len(features) > 0 {
		args = append(args, "--features", strings.Join(features, ","))
	}
	return strings.Join(args, " ")
}

func (m *model) clearOptions(items *[]option) {
	for i := range *items {
		(*items)[i].Selected = false
	}
}

func (m *model) selectAll(items *[]option) {
	for i := range *items {
		(*items)[i].Selected = true
	}
}

func (m *model) toggleAll(items *[]option) {
	shouldSelect := !allSelected(*items)
	for i := range *items {
		(*items)[i].Selected = shouldSelect
	}
}

func (m *model) setEvalByValue(value string) {
	for i := range m.evals {
		m.evals[i].Selected = m.evals[i].Value == value
	}
	if len(m.evals) > 0 && !anySelected(m.evals) {
		m.evals[0].Selected = true
	}
}

func discoverEvalOptions(bins []cargoBin) []option {
	options := make([]option, 0)
	for _, bin := range bins {
		if !strings.HasPrefix(bin.Name, "oopsmate-") {
			continue
		}
		slug := strings.TrimPrefix(bin.Name, "oopsmate-")
		options = append(options, option{Label: prettyEvalName(slug), Value: bin.Name})
	}
	sort.Slice(options, func(i, j int) bool { return options[i].Label < options[j].Label })
	return options
}

func discoverFeatureOptions(features map[string][]string, names []string) []option {
	options := make([]option, 0, len(names))
	for _, name := range names {
		if _, ok := features[name]; ok {
			options = append(options, option{Label: name, Value: name})
		}
	}
	return options
}

func prettyEvalName(slug string) string {
	switch slug {
	case "pesto":
		return "PeSTO"
	case "nnue":
		return "StockfishNnue"
	default:
		parts := strings.Split(slug, "-")
		for i, part := range parts {
			if part == "" {
				continue
			}
			parts[i] = strings.ToUpper(part[:1]) + part[1:]
		}
		return strings.Join(parts, "")
	}
}

func loadCargoManifest(path string) (cargoManifest, error) {
	var manifest cargoManifest
	if _, err := toml.DecodeFile(path, &manifest); err != nil {
		return cargoManifest{}, fmt.Errorf("failed to parse %s: %w", path, err)
	}
	return manifest, nil
}

func findRepoRoot() (string, error) {
	wd, err := os.Getwd()
	if err != nil {
		return "", err
	}
	cur := wd
	for {
		cargoPath := filepath.Join(cur, "Cargo.toml")
		if _, err := os.Stat(cargoPath); err == nil {
			return cur, nil
		}
		parent := filepath.Dir(cur)
		if parent == cur {
			break
		}
		cur = parent
	}
	return "", fmt.Errorf("could not find repo root from %s", wd)
}

func copyExecutable(src, dst string) error {
	if err := os.MkdirAll(filepath.Dir(dst), 0o755); err != nil {
		return err
	}
	in, err := os.Open(src)
	if err != nil {
		return err
	}
	defer in.Close()

	info, err := in.Stat()
	if err != nil {
		return err
	}

	out, err := os.Create(dst)
	if err != nil {
		return err
	}
	defer out.Close()

	if _, err := io.Copy(out, in); err != nil {
		return err
	}
	return os.Chmod(dst, info.Mode())
}

func sanitizeOutputName(name string) string {
	name = strings.TrimSpace(name)
	name = strings.ReplaceAll(name, " ", "-")
	name = strings.Map(func(r rune) rune {
		switch {
		case r >= 'a' && r <= 'z':
			return r
		case r >= 'A' && r <= 'Z':
			return r + ('a' - 'A')
		case r >= '0' && r <= '9':
			return r
		case r == '-' || r == '_':
			return r
		default:
			return -1
		}
	}, name)
	return strings.Trim(name, "-_")
}

func countSelected(items []option) int {
	count := 0
	for _, item := range items {
		if item.Selected {
			count++
		}
	}
	return count
}

func allSelected(items []option) bool {
	return len(items) > 0 && countSelected(items) == len(items)
}

func anySelected(items []option) bool {
	return countSelected(items) > 0
}

func ternary[T any](cond bool, a, b T) T {
	if cond {
		return a
	}
	return b
}

func min(a, b int) int {
	if a < b {
		return a
	}
	return b
}

func max(a, b int) int {
	if a > b {
		return a
	}
	return b
}
