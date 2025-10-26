
# ♟️ Oops!Mate

```bash
        ,....,
      ,::::::<
     ,::/^\"``.
    ,::/, `   e`.
   ,::; |        '.
   ,::|  \___,-.  c)
   ;::|     \   '-'
   ;::|      \
   ;::|   _.=`\
   `;:|.=` _.=`\
     '|_.=`   __\
     `\_..==`` /
      .'.___.-'.
     /          \
    /'--......--'\
    `"--......--"
```

**A modular, high-performance (hopefully) chess engine being written in Rust.**  
Building from the ground up with bitboards, magic, and good ol’ brainpower ( and some AI ... a good some of AI, I cant be reading all the docs myself, we arent in the stone ages anymore).

---

## 🧩 Design Goals

- **Modularity** – each crate has one job and does it well
- **Performance** – bitboards, magic tricks, and minimal allocations
- **Clean code** – ergonomic, idiomatic Rust
- **Extensibility** – easy to tweak, add heuristics, or swap components

---

## 🧪 Testing

All crates will be testable independently:
```bash
cargo test -p magics # test magic bitboards
cargo test -p board # test game state logic
# etc...
```
Worspace wide:
```bash
cargo test --workspace
```

---

## 🚧 WIP
The engine is fully functional, but it’s still under active development with many enhancements to come. 
