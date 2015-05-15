ANNIS4
======

This is a prototype for a new backend implementation of the ANNIS linguistic search and visualization system (http://github.com/korpling/ANNIS/). 
It is part of my ongoing thesis and while there are test cases it is **highly experimental code**!

**Don't use this for real linguistic research!**

There will be a technical report that describes how this prototype works.

How to compile
---------------

ANNIS4 has several dependencies which need to be installed (and some which are already delivered with the source).
Currently ANNIS4 is only tested under the Ubuntu 15.04 operating system,
but it is planned to add offical support for further operating systems.

The build process is not as clean and easy as it should be yet, but that's why it's a prototype.  

1. get the code and change the working directory to the source folder: `cd <annis4code>`
2. install CMake build system (at least version XXX): `sudo apt-get install cmake`
3. install dependencies: `sudo apt-get install build-essential libicu-dev libre2-dev libboost-dev libboost-system-dev libboost-filesystem-dev libboost-serialization-dev libncurses5-dev`
4. create a build-directory: `mkdir build && cd build`
5. build: `cmake ../ && make`
6. optionally run the tests if you have the necessary corpus data installed: `./test_ANNIS4`
