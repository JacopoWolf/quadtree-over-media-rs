<h1 align="center">Quadtree over media</h1>
<p align="center">
<img src="https://img.shields.io/github/v/tag/jacopowolf/quadtree-over-media-rs?include_prereleases&label=latest&sort=semver"/>
<img src="https://img.shields.io/github/languages/top/jacopowolf/quadtree-over-media-rs?logo=rust&color=brown" /><br>
<a href="https://github.com/JacopoWolf/quadtree-over-media-rs/wiki">Documentation</a>
</p>


---

This program calculates and draws "quads" on images in varius input formats (for supported formats see the [image crate](https://crates.io/crates/image)).

![YouTube Video Views](https://img.shields.io/youtube/views/G434WPz8MRk?style=social)
[demonstration video](https://youtu.be/G434WPz8MRk) 


Everything is completely customizable, from rgba tresholds to subdivide the quads to how to draw the quads.

### Examples

Examples use this CC image from wikipedia https://commons.wikimedia.org/wiki/File:Rainbow_in_Budapest.jpg

Below examples all add parameters to this base command:

`quadtree-over-media -i Rainbow_in_Budapest.jpg -o rainbow-something.jpg`


* <code>--fill</code><br>
<img src="docs/rainbow-fill.jpg" width="300">

* <code>--fill-with dog-whitebg.jpg</code><br>
<img src="docs/rainbow-dog-nofill.jpg" width="300">

* <code>--fill --fill-with dog-whitebg.jpg</code><br>
<img src="docs/rainbow-dog.jpg" width="300">

* <code>--fill --fill-with dog-whitebg.jpg --treshold 000</code><br>
<img src="docs/rainbow-dog-t000.jpg" width="300">

### Planned features
* further optimization
* take directly a video as input [improbable]
* shared image cache for batching
* separate into a CLI bin and a library

### Inspired by
* https://github.com/snailcon/QuadtreeAmogufier
