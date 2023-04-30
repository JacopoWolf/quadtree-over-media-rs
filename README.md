<div align="center">
<h1>quadtree over media</h1>
<img src="https://img.shields.io/badge/built_using-Rust-F46623?style=for-the-badge&logo=rust" />
<img src="https://img.shields.io/badge/built_with-♥-FF69B4?style=for-the-badge" />
<br/>
<img src="https://img.shields.io/github/v/tag/jacopowolf/quadtree-over-media-rs?include_prereleases&label=latest&sort=semver&style=flat-square"/>
<hr>
<a href="https://github.com/JacopoWolf/quadtree-over-media-rs/wiki">Documentation 📗</a>
<br>
<code>quadtree-over-media --help</code>
</div>

---

This program calculates and draws "quads" on images in variuos input formats (for supported formats see the [image crate](https://crates.io/crates/image)).

[![YouTube Video Views](https://img.shields.io/youtube/views/G434WPz8MRk?style=social)
demonstration video](https://youtu.be/G434WPz8MRk) 

Everything is completely customizable, from rgba tresholds to subdivide the quads to how to draw the quads.

## Installation
For the latest tag check the release page

`cargo install --git https://github.com/JacopoWolf/quadtree-over-media-rs.git --tag v1.0.0`

_NOTE: I don't suggest installing directly from the base branch (by not specifying a tag) because I'm lazy and develop directly in main_

## Examples

Below examples all add parameters to this base command:

`quadtree-over-media -i Rainbow_in_Budapest.jpg -o rainbow-something.jpg`

|                                                     |                                                |                                                         |
| :-------------------------------------------------: | :--------------------------------------------: | :-----------------------------------------------------: |
|                 __`--color black`__                 |       __`--color blue --treshold FF0`__        |                      __`--fill`__                       |
|  <img src="docs/rainbow-simple.jpg" width="300" >   | <img src="docs/rainbow-blue.jpg" width="300" > |     <img src="docs/rainbow-fill.jpg" width="300" >      |
|              __`--fill-with dog.jpg`__              |        __`--fill --fill-with dog.jpg`__        | __`--fill --fill-with dog.jpg` <br/> `--treshold 000`__ |
| <img src="docs/rainbow-dog-nofill.jpg" width="300"> |  <img src="docs/rainbow-dog.jpg" width="300">  |    <img src="docs/rainbow-dog-t000.jpg" width="300">    |

> CC image from Wikipedia https://commons.wikimedia.org/wiki/File:Rainbow_in_Budapest.jpg


### Inspired by
* https://github.com/snailcon/QuadtreeAmogufier
