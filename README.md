kdlize
-----

Provides helpful utilities for interfacing with [kdl](https://github.com/kdl-org/kdl) / [kdl-rs](https://github.com/kdl-org/kdl-rs) structures.
Notable additions include:
- traits for parsing KDL to a user-defined type (`FromKdl`) and building KDL data from a user-defined type (`AsKdl`)
- Node reading API; parsing specific types, tracking what positional argument was last consumed, navigating to a child node
- Node building API; making new kdl nodes from primitive types or user structs

KdlValue
	String String
		String, str, char, ToString/FromStr
	Integer i128
		i128, u128, i64, u64, i16, u16, i8, u8, usize, isize
	Float f64
		f64, f32
	Bool bool
		bool
	Null
		Option::None

Values (always To/From KdlValue)
Properties (all To/From KdlValue, and supports Option)
To/From KdlNode
Option types with OmitIfEmpty
