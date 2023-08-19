# Harmony

The goal of this project is to experiment building a decentralized, privacy/security conscious, moddable, media exploration/creation tool (that can also run in your web browser!)

Decentralized social protocols like bluesky's at, lens, minds and countless awful social nft/metaverse/crypto projects attempt to completely reinvent the wheel and in my opinion fail miserably at it. They suffer from adoption issues specifically because of their lack of interoperability with existing file formats, protocols and services. Furthermore they remain products for the enthusiast crowd, and are not accessible to (or useful for) the average user.

The advertising space has proven time and time again that users don't care about ads, privacy, or security. They care about having convenience and features for free. With Harmony, I attempt to bridge the gap between decentralized technologies that have been historically difficult to use and provide a dead-simple experience for users that is just as accessible (or more accessible) than the traditional web.

I see many exciting trends towards more open communication standards (such as matrix) and collaborative mindmapping based tools (miro, obsidian canvas, mental canvas, and more). And thus one of the aims of Harmony will be to provide an open format on top of the gltf standard that provides a similarly rich experience, but with an extreme focus on the experience of those exploring artwork (think assisted technologies, web, mobile, desktop, vr and more). [My ideas are largely inspired by Scott McCloud's infinite canvases.](https://guild.art/blog/a-letter-to-scott-mccloud#what-im-working-on)

Harmony's aim is to do some or all the following eventually:

- Use the [IPFS](https://ipfs.tech/) protocol to distribute files/media between peers
- Use the [Freenet](https://freenet.org/) protocol to provide real-time communication, syncing and trust/authentication mechanisms between peers
    - Media tagging, filtering and search functionality
    - Receive live notification of updates from your favorite creators
    - Share private, encrypted content with select people
- Use the [Bevy](https://bevyengine.org/) game engine to render multimedia content, including rich text, 3D graphics, video, audio and more
- Use [Wgpu](https://wgpu.rs/) as a portable graphics backend allowing Harmony to run on any device including inside web browsers
- Allow editing/viewing of any file format, including 3D models, video, audio, images, markdown, etc
- Have a rich catalogue of optional addons/plugins called [Elements](https://github.com/MarcGuiselin/hmny/tree/main/elements/#readme)
    - Elements add functionality to the browser, such as parsing an unknown mimetype, rendering a new type of media, adding user authentication, etc
    - Internally these are [Wasm](https://webassembly.org/) modules that can be easily distributed/downloaded
    - Harmony will have a built-in package manager that uses signatures from trusted publishers to verify integrity and authenticity of elements and automatically download updates
- Host a server to enable users to browse from a web browser or devices with latency/bandwidth constraints such as mobile

These are extremely lofty goals, and I don't expect to achieve them all. My hope is to build an platform using [Elements](https://github.com/MarcGuiselin/hmny/tree/main/elements/#readme) that is extensible enough to allow others to add functionality where they think it is needed.

## Development

Rust was the obvious choice for this project. First because it's a [pretty neat programming language](https://www.youtube.com/@NoBoilerplate) and it's backed by an amazing community. But the gist of it really boils down to the following:
- I'm not interested writing more typescript as I already do that as a job
- I care about results. And rust allows me to write type-safe and thread-safe code confidently (if a bit slowly because I'm still learning haha)
- I want to be able to run natively and at high performance on any device, not just inside a web browser
- I've been experimenting doing graphical things that are [very difficult to do on the traditional web](https://stackoverflow.com/questions/72008951) so it necessitates lower-level graphics programming
- Since I'm no graphics engineer, don't have the time to build my own graphics engine and have a strong dislike for the DX of working with javascript-based 3d graphics libraries, Bevy's provided a fantastic starting point while allowing me to drop down to lower-level graphics APIs when needed
- Freenet ([Locutus](https://github.com/freenet/locutus)) and many of the tools/libraries I'd like to use are already built in rust

### Run Harmony Browser 

Run the following command:

```sh
cargo run
```

### How to build elements

Run the following command:

```sh
cargo build --workspace --exclude hmny --target wasm32-unknown-unknown -r
```
