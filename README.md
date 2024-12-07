# rIC3 Hardware Model Checker

[![Crates.io](https://img.shields.io/crates/v/rIC3.svg)](https://crates.io/crates/rIC3)
[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)

[[HWMCC'24](https://hwmcc.github.io/2024/)] rIC3 won *1<sup>st</sup>* place in the bit-level track and the word-level without array track at the *Hardware Model Checking Competition* (HWMCC) 2024
<p align="center">
	<img width="250" height="auto" src="./images/hwmcc24_aiger.png" style="display:inline-block;">
	<img width="250" height="auto" src="./images/hwmcc24_btor2_bv.png" style="display:inline-block;">
</p>

To view the submission for HWMCC'24, please checkout the `HWMCC24` branch or download the binary release at https://github.com/gipsyh/rIC3-HWMCC24.

## Build and Run
Currently, some dependency libraries are linked through pre-compiled static files in the repository, and they have a dependency on the glibc version. Ubuntu 20.04 or later works fine.

- Install the Rust compiler https://www.rust-lang.org/
- Switch to nightly ````rustup default nightly````
- ````git clone --recurse-submodules https://github.com/gipsyh/rIC3````
- ````cargo r --release -- <aig model>````

Copyright (C) 2023 - Present, Yuheng Su (gipsyh.icu@gmail.com). All rights reserved.

Without obtaining authorization, rIC3 is not allowed to be used for any commercial purposes.
