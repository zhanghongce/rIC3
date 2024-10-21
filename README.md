# rIC3

rIC3 model checker.

> Copyright (c) 2023 - Present by Yuheng Su [(gipsyh.icu@gmail.com)](gipsyh.icu@gmail.com), Qiusong Yang [(qiusong@iscas.ac.cn)](qiusong@iscas.ac.cn) and  Yiwei Ci [(yiwei@iscas.ac.cn)](yiwei@iscas.ac.cn)

[[HWMCC'24](https://hwmcc.github.io/2024/)] rIC3 won *1<sup>st</sup>* place in BV track at the prestigious *Hardware Model Checking Competition* (HWMCC) 2024
<p align="center">
	<img align="center" width="400" height="auto" src="./images/hwmcc24_aiger.png"></img>
</p>

<p align="center">
	<img align="center" width="400" height="auto" src="./images/hwmcc24_btor2_bv.png"></img>
</p>

To view the submission for HWMCC'24, please checkout the `HWMCC24` branch or download the binary release at https://github.com/gipsyh/rIC3-HWMCC24.

## Build and Run
- Install the Rust compiler https://www.rust-lang.org/
- ````git clone --recurse-submodules https://github.com/gipsyh/rIC3````
- ````cargo r --release -- <aig model>````
