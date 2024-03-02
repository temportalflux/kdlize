kdlize
-----

Provides helpful utilities for interfacing with [kdl](https://github.com/kdl-org/kdl) / [kdl-rs](https://github.com/kdl-org/kdl-rs) structures.
Notable additions include:
- traits for parsing KDL to a user-defined type (`FromKdl`) and building KDL data from a user-defined type (`AsKdl`)
- Node reading API; parsing specific types, tracking what positional argument was last consumed, navigating to a child node
- Node building API; making new kdl nodes from primitive types or user structs
