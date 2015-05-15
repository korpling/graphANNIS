annis4
======

ANNIS4

How to compile
---------------

ANNIS4 has several dependencies which need to be installed.
Currently ANNIS4 is only tested under the Ubuntu operating system,
but it is planned to add offical support for further operating systems.

1. get the code and change working directory to the source folder: `cd <annis4code>`
2. install CMake build system (at least version XXX): `sudo apt-get install cmake`
3. install dependencies: `sudo apt-get install XXX`
4. create a build-directory: `mkdir build && cd build`
5. build: `cmake ../ && make`
6. optionally run the tests: `./test_ANNIS4`
