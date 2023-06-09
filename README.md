# ola

[![Documentation](https://img.shields.io/docsrs/ola)](https://docs.rs/ola)
[![Crates.io](https://img.shields.io/crates/v/ola)](https://crates.io/crates/ola)

`ola` is a client for interacting with the [Open Lighting Architecture] API.
`ola` provides a basic, synchronous client, and a more capable asynchronous
client built on [Tokio].

[Open Lighting Architecture]: https://www.openlighting.org/ola/
[Tokio]: https://tokio.rs/

## Building

This crate uses Protobuf implementations generated by [Prost], and by
extention requires `protoc` to build. To get `protoc`, see the Protobuf
[install instructions].

[Prost]: https://github.com/tokio-rs/prost
[install instructions]: https://github.com/protocolbuffers/protobuf#protocol-compiler-installation

<br />

#### License

<sup>
Copyright (C) Jared Beller, 2023.
</sup>
<br />
<sup>
Released under the <a href="https://www.gnu.org/licenses/old-licenses/lgpl-2.1.txt">GNU Lesser General Public License, Version 2.1</a> or later. See <a href="LICENSE">LICENSE</a> for more information.
</sup>
