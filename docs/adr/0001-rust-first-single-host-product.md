# Build a Rust-first SDK around one resident host

The first supported product is one process-lifetime ASI Host, versioned C ABI Services, and safe Rust facades for one exact GTA SA 1.0 US build and the listed SA-MP builds. C/C++ SDK support, managed plugin loading, rendering, CLEO, and compatibility layers are later work; coexistence with third-party hook owners is best-effort and has no first-release guarantee. This boundary keeps one component responsible for process state and hooks while the public product remains focused on Rust plugin authors.
