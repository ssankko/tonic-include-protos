Include all generated proto server and client items.

```rust
tonic_include_protos::include_protos!();
```

# Motivation:
I am working on a project with somewhat large grpc base, where we used [tonic] 
and google apis. [tonic] uses nice [prost]-based build system, but it generates
a lot of files. Each of them must be included manually in separate `mod` to work
correctly. If you just smash them all in one module, compilation will likely fail.

# How it works:
This macro will construct mod tree based on grpc package names.
For example if you have two generated by [tonic_build] files in [`OUT_DIR`]:
 - `package.api.example.rs` 
 - `package.api.another_example.rs`

The result will be equivalent to this:

```rust
pub mod package{
    pub mod api {
        pub mod example {
            include!(concat!(env!("OUT_DIR"), "/google.api.example.rs"));
        }
        pub mod another_example {
            include!(concat!(env!("OUT_DIR"), "/google.api.another_example.rs"));
        }
    }
}
```

If [`OUT_DIR`] won't work for you (when you set [tonic_build] to save files
in other directory or for some other reason) you can set `TIP_OUT_DIR` environment variable
to point on the directory you need.

I know this solution is not perfect, but it's getting work done. If you have better ideas on implementation - feel free to open issue or PR.

# License
This project is licensed under the [MIT license](https://opensource.org/licenses/MIT).

[`OUT_DIR`]: https://doc.rust-lang.org/cargo/reference/environment-variables.html#environment-variables-cargo-sets-for-build-scripts#[proc_macro]
[tonic_build]: https://docs.rs/tonic-build
[tonic]: https://docs.rs/tonic
[prost]: https://docs.rs/prost