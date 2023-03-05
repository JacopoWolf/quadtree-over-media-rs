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


### Examples

Examples use this CC image from wikipedia https://commons.wikimedia.org/wiki/File:Rainbow_in_Budapest.jpg

Below examples all add parameters to this base command:

`quadtree-over-media -i Rainbow_in_Budapest.jpg -o rainbow-something.jpg`


* <code>--fill</code><br>
<img src="docs/rainbow-fill.jpg" width="400">

* <code>--fill-with dog-whitebg.jpg</code><br>
<img src="docs/rainbow-dog-nofill.jpg" width="400">

* <code>--fill --fill-with dog-whitebg.jpg</code><br>
<img src="docs/rainbow-dog.jpg" width="400">

* <code>--fill --fill-with dog-whitebg.jpg --treshold 000</code><br>
<img src="docs/rainbow-dog-t000.jpg" width="400">


### Inspired by
* https://github.com/snailcon/QuadtreeAmogufier
