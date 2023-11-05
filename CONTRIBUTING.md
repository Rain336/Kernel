# Contributing to Microdragon

First up, thanks a lot for wanting to contribute! It means a lot!
Currently there is not a lot of documentation on how to contribute, but I'll try my best to expand it as good as possible.
So for now the best way to find out how things are done, is to dig through the code or asking me, I guess.
I'll also try to set up some community places and link them here to make it easier to ask me about things, if needed.

## Rough Overview

Microdragon is made up of independent kernel modules.
The final goal is for them to be pluggable, so you can disable some, replace some with your own,
have a fully working OS kernel or strip it down to a hello world kernel.
All modules follow a common interface defined in the `modules/interface` crate.
The interface supplies information about the system, mainly provided by the bootloader.
To collect this info a bootloader interface crate is needed,
they represent the entry point of the kernel and start the runner module.
Currently only the limine bootloader is supported.
