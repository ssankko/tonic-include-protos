extern crate proc_macro;
use proc_macro::TokenStream;
use std::collections::HashMap;

#[derive(Debug)]
enum TreeEntry {
    Node(String),
    Branch(HashMap<String, Box<TreeEntry>>),
}

impl TreeEntry {
    fn get_mut(&mut self, part: &str) -> Option<&mut Box<TreeEntry>> {
        match self {
            TreeEntry::Branch(tree) => tree.get_mut(part),
            _ => panic!(),
        }
    }

    fn get(&mut self, part: &str) -> Option<&Box<TreeEntry>> {
        match self {
            TreeEntry::Branch(tree) => tree.get(part),
            _ => panic!(),
        }
    }

    fn insert(&mut self, part: String, node: TreeEntry) {
        match self {
            TreeEntry::Branch(tree) => {
                tree.insert(part, Box::new(node));
            }
            _ => panic!(),
        }
    }
}

/// Include all generated proto server and client items.
///
/// ```rust
/// tonic_include_protos::include_protos!();
/// ```
/// ---
///
/// This macro will construct mod tree based on grpc package names.
/// For example if you have two generated by [tonic_build] files in [`OUT_DIR`]:
/// `package.api.example.rs` and  `package.api.another_example.rs`
/// result will look like this:
///
/// If ['OUT_DIR'] won't work for you (when you set [tonic_build] to save files
/// in other directory or for some other reason) you can set 'TIP_OUT_DIR' environment variable
/// to point on the directory you need
///
/// ```rust
/// pub mod package{
///     pub mod api {
///         pub mod example {
///             include!(concat!(env!("OUT_DIR"), "/google.api.example.rs"));
///         }
///         pub mod another_example {
///             include!(concat!(env!("OUT_DIR"), "/google.api.another_example.rs"));
///         }
///     }
/// }
/// ```
/// [`OUT_DIR`]: https://doc.rust-lang.org/cargo/reference/environment-variables.html#environment-variables-cargo-sets-for-build-scripts#[proc_macro]
/// [tonic_build]: https://docs.rs/tonic-build
#[proc_macro]
pub fn include_protos(_item: TokenStream) -> TokenStream {
    let out_dir = std::env::var("TIP_OUT_DIR")
        .or(std::env::var("OUT_DIR"))
        .unwrap();
    let files = std::fs::read_dir(&out_dir).unwrap();
    // extract file names from output directory
    let file_names = files
        // prost constructs file names based on a grpc package name, which
        // in turn must be valid utf-8 identifier, so i use to_string_lossy fearlessly
        .map(|x| x.unwrap().file_name().to_string_lossy().to_string());

    // --------
    // traverse all files and construct tree-like structure of namespaces
    // --------
    let mut tree = TreeEntry::Branch(Default::default());
    for file_name in file_names {
        let mut current_branch = &mut tree;
        // split names by dot.
        // `tonic_build` uses dots to represent namespaces
        // for example google.logging.v2.rs will become
        // [google, logging, v2, rs]
        for part in file_name.split('.') {
            if part == "rs" {
                *current_branch = TreeEntry::Node(file_name.to_string());
                continue;
            }

            if let None = current_branch.get(part) {
                current_branch.insert(part.to_owned(), TreeEntry::Branch(Default::default()));
            }
            current_branch = current_branch.get_mut(part).unwrap();
        }
    }
    // --------

    // simple recursive function to construct mod tree based on a
    // tree built earlier
    fn construct(tree_entry: Box<TreeEntry>, result: &mut String, out_dir: &str) {
        match *tree_entry {
            TreeEntry::Node(node) => {
                result.push_str(&format!(r#"include!("{}/{}");"#, out_dir, node));
            }
            TreeEntry::Branch(branch) => {
                for (name, child) in branch {
                    result.push_str(&format!("pub mod {} {{", name));
                    construct(child, result, out_dir);
                    result.push_str("}");
                }
            }
        }
    };

    let mut result = String::new();
    construct(Box::new(tree), &mut result, &out_dir);
    result.parse().unwrap()
}
