# gencomments
A Rust library that provides the functionality needed to parse Rust source code, to modify it programmatically, and to 
create the modified source text.

There are a number of Rust crates that serialize and deserialize source code as an intermediate step in the compile process,
but I have been unable to find any crate that provides the functionality to modify Rust source code and to output it in
text format while retaining all of the formatting that is included in the original source code. For example, the syn and serde_syn
crates can parse a text file and generate a text representation that is useful to a compiler, but it is not possible to then
take that representation and regenerate the original source text.

gencomments is an attempt to generate a library that parses source text, allow it to be modified, and to create the modified 
source text. Without using any of the modification functionality, the gencomments library should be able to generate new source
code that is identical to the original source.
