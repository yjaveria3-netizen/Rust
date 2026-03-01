# Rust Concepts — Complete Reference

A comprehensive guide to Rust programming language concepts, with explanations (`.md`) and code examples (`.rs`) for each topic.

---

## Concepts Covered

| # | Concept | Explanation | Code Example |
|---|---------|-------------|--------------|
| 01 | [Variables & Data Types](01_variables_and_data_types.md) | Immutability, shadowing, scalar/compound types | [01_variables_and_data_types.rs](01_variables_and_data_types.rs) |
| 02 | [Ownership](02_ownership.md) | Rules of ownership, move vs copy, drop | [02_ownership.rs](02_ownership.rs) |
| 03 | [References & Borrowing](03_references_and_borrowing.md) | &T, &mut T, borrow rules, slices | [03_references_and_borrowing.rs](03_references_and_borrowing.rs) |
| 04 | [Structs](04_structs.md) | Structs, methods, impl blocks, derive | [04_structs.rs](04_structs.rs) |
| 05 | [Enums & Pattern Matching](05_enums_and_pattern_matching.md) | Enums, Option, Result, match, if let | [05_enums_and_pattern_matching.rs](05_enums_and_pattern_matching.rs) |
| 06 | [Traits](06_traits.md) | Defining/implementing traits, dyn Trait, operator overloading | [06_traits.rs](06_traits.rs) |
| 07 | [Generics](07_generics.md) | Generic functions/structs/enums, monomorphization, lifetimes | [07_generics.rs](07_generics.rs) |
| 08 | [Error Handling](08_error_handling.md) | panic!, Result, ?, custom errors | [08_error_handling.rs](08_error_handling.rs) |
| 09 | [Closures & Iterators](09_closures_and_iterators.md) | Fn/FnMut/FnOnce, iterator adapters/consumers | [09_closures_and_iterators.rs](09_closures_and_iterators.rs) |
| 10 | [Smart Pointers](10_smart_pointers.md) | Box, Rc, RefCell, Arc, Mutex | [10_smart_pointers.rs](10_smart_pointers.rs) |
| 11 | [Concurrency](11_concurrency.md) | Threads, channels, Mutex, RwLock, atomics | [11_concurrency.rs](11_concurrency.rs) |
| 12 | [Collections](12_collections.md) | Vec, HashMap, HashSet, BTreeMap, VecDeque, BinaryHeap | [12_collections.rs](12_collections.rs) |
| 13 | [Lifetimes](13_lifetimes.md) | Lifetime annotations, elision, 'static, structs | [13_lifetimes.rs](13_lifetimes.rs) |
| 14 | [Modules & Crates](14_modules_and_crates.md) | mod, pub, use, visibility, file organization | [14_modules_and_crates.rs](14_modules_and_crates.rs) |
| 15 | [Control Flow](15_control_flow.md) | if, loop, while, for, match, let..else | [15_control_flow.rs](15_control_flow.rs) |

---

## Learning Order

For beginners, follow the numbered order — each concept builds on the previous ones:

```
Variables → Ownership → References → Structs → Enums
    ↓
Traits → Generics → Error Handling → Closures/Iterators
    ↓
Smart Pointers → Concurrency → Collections → Lifetimes → Modules
```

---

## Running the Examples

Each `.rs` file is a standalone program. To run one:

```bash
# Run directly
rustc 01_variables_and_data_types.rs && ./variables_and_data_types

# Or better, with Cargo:
cargo new my_rust_learning
# Then paste the content into src/main.rs
cargo run
```

---

## Key Concepts Summary

**Memory Safety Without GC**  
Rust achieves memory safety through ownership, borrowing, and lifetimes — all enforced at compile time with no runtime overhead.

**Zero-Cost Abstractions**  
Traits, generics, and iterators compile down to code as efficient as hand-written C. You don't pay for what you don't use.

**Fearless Concurrency**  
The ownership and type system prevents data races at compile time, making concurrent code safe to write.

**Expressive Type System**  
Enums with data, pattern matching, `Option`/`Result`, traits, and generics combine for highly expressive, safe code.
