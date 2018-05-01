[![Build Status](https://travis-ci.org/thomaskrause/graphANNIS.svg?branch=develop)](https://travis-ci.org/thomaskrause/graphANNIS)

graphANNIS
==========

This is a prototype for a new backend implementation of the ANNIS linguistic search and visualization system (http://github.com/korpling/ANNIS/). 

It is part of my ongoing thesis and while there are test cases it is **highly experimental code and it is not ready to be used by end-users yet**!

How to compile
---------------

graphANNIS has several dependencies which need to be installed (and some which are already delivered with the source).
Currently graphANNIS is only tested under the Ubuntu 16.04 operating system,
but it is planned to add offical support for further operating systems.

The build process is not as clean and easy as it should be yet, but that's why it's a prototype.  

1. get the code and change the working directory to the source folder: `cd <graphanniscode>`
2. install CMake build system: `sudo apt-get install cmake`
3. install dependencies: `sudo apt-get install build-essential libicu-dev libboost-dev libboost-system-dev libboost-filesystem-dev libboost-thread-dev libncurses5-dev`
4. create a build-directory: `mkdir build && cd build`
5. build: `cmake ../ && make`
6. optionally run the tests if you have the necessary corpus data installed: `./test_ANNIS4`

3rd party dependencies
----------------------

This software depends on several 3rd party libraries. These are documented in the BOM.txt file in this folder.

Author(s)
---------

* Thomas Krause (thomaskrause@posteo.de)
