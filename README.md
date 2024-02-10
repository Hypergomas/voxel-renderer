<div align="center">

# Voxel renderer

</div>

Welcome to my voxel renderer repo! Here, my experiments on building a voxel renderer from scratch will be stored in the Git branches.

## The motivation

I've always been a fan of voxel games, especially the ones with modifiable terrain. Games like Minecraft and Deep Rock Galactic use voxel techniques in order to add a greater degree of immersion and interactivity that I think is unparalleled by other kinds of games.

It's actually been a couple years (circa 2022) since I've wanted to build something like this, and I've actually been saving screenshots, blog posts, articles and YouTube videos about it for quite a while.

Either way, the main inspiration for this project has definitely been playing a pretty bare bones homebrew Minecraft modpack I made centered around Create. I downloaded some ambience mods and shaders to go alongside it and man, is it a beautiful experience. It made me question how much better the graphics could get, though, and since then I've been trying out different techniques to make smooth, modifiable voxel terrain that looks and feels good.

## The toolkit

For this project, I've been using the Rust programming language to make the whole thing. It's what I'm most comfortable with, since I have over 30 (private) projects I made for learning purposes over the course of the last couple years. Not only that, but the language has a guaranteed minimum performance margin and a great ecosystem for this project, including the amazing [`wgpu`](https://github.com/gfx-rs/wgpu)!

I chose `wgpu` because it allows me to write platform-independent code while also supporting many great features of the best graphics APIs, like compute shaders, which I plan on using extensively.

## The results

So far, my best results have been achieved using a traditional voxel renderer that can only do hard voxels (cubes). You can check them out on the `hard-voxels` branch!

## Trying it yourself

As this project is built with the [Rust](https://www.rust-lang.org/) language, you will need Cargo in order to compile and run the code. Instructions for installing the Rust toolchain can be found at https://rustup.rs/.

Every iteration of the renderer is built to compile and run with the default Cargo commands. To build, simply use `cargo build`. To run, simply use `cargo run`.

## Contributions

This repo is just a hobby project of mine, a pet experiment of sorts. As such, while you are free to fork and use this code and it's assets to your heart's content, there is no need for committing or creating pull requests :)

## Licensing

All code and assets in this repository are licensed under the [Unlicense](https://unlicense.org/) unless explicitly stated otherwise.