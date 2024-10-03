# [Profile-Guided Optimization (PGO)](https://en.wikipedia.org/wiki/Profile-guided_optimization)

PGO is a technique to automatically improve a program's runtime performance based on profiling.

PGO should be applied to the final executable, it cannot be applied to the Rust crate, nor is a pre-compiled static library provided. If you want to benefit from PGO for maximum performance, you will need to follow the steps below.

There is various ways to do this, but in general, it follows these steps:

1. Create a set of realistic data/examples/benchmarks. PGO needs to be primed with profiling, using unrealistic data can degrade performance in production, even if performance increases in the test cases.
2. Compile/build an instrumented executable.
3. Run instrumentation to collect profile data.
4. Compile/build the final executable given the profile data with PGO enabled optimizations.

PGO can be time consuming and results vary. Always test the improvements on your use-case to see whether it helps and how much.

## Applying PGO

[Rust provides a guide on the use of PGO](https://doc.rust-lang.org/rustc/profile-guided-optimization.html) and there is a nice tool, [cargo pgo](https://github.com/Kobzol/cargo-pgo), to simplify application of PGO.

Using this tool, building a PGO-optimized executable is as simple as:

```bash
# Build the instrumented executable.
cargo pgo instrument build
# Run the instrumented binary to produce profiles.
cargo pgo instrument run
# Build the optimized executable.
cargo pgo optimize
```

Instrumentation can also be applied to tests and benchmarks. See the [documentation of cargo pgo](https://github.com/Kobzol/cargo-pgo) for further information.

## Expected Results

[Results on benchmarks of this library](https://github.com/FlixCoder/serde-brief/issues/5) have shown very noticable improvements in performance. There is varying speed-ups, but realisticly I would guess it is about 30% improvements, currently (Version 0.1.0).

## Further Resources

- [Various PGO results & resources](https://github.com/zamazan4ik/awesome-pgo)
