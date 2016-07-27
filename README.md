graphANNIS
==========

This is a prototype for a new backend implementation of the ANNIS linguistic search and visualization system (http://github.com/korpling/ANNIS/). 
It is part of my ongoing thesis and while there are test cases it is **highly experimental code**!

**Don't use this for real linguistic research!**

There will be a technical report that describes how this prototype works.

How to compile
---------------

graphANNIS has several dependencies which need to be installed (and some which are already delivered with the source).
Currently graphANNIS is only tested under the Ubuntu 16.04 operating system,
but it is planned to add offical support for further operating systems.

The build process is not as clean and easy as it should be yet, but that's why it's a prototype.  

1. get the code and change the working directory to the source folder: `cd <graphanniscode>`
2. install CMake build system: `sudo apt-get install cmake`
3. install dependencies: `sudo apt-get install build-essential libicu-dev libre2-dev libboost-dev libboost-system-dev libboost-filesystem-dev libboost-serialization-dev libncurses5-dev`
4. create a build-directory: `mkdir build && cd build`
5. build: `cmake ../ && make`
6. optionally run the tests if you have the necessary corpus data installed: `./test_ANNIS4`

3rd party dependencies
----------------------

This software depends on libraries and code of the following projects:

* Boost - Copyright by the various Boost Contributors: Boost Software License (http://www.boost.org/LICENSE_1_0.txt)
* Celero C++ Benchmarking Library - Copyright by John Farrier, Hellebore Consulting LLC: Apache License, Version 2.0 (http://www.apache.org/licenses/LICENSE-2.0)
* Google C++ B-tree - Copyright by Google Inc.: Apache License, Version 2.0 (http://www.apache.org/licenses/LICENSE-2.0)
* Google Test - Copyright by Google Inc.: New BSD License (http://opensource.org/licenses/BSD-3-Clause)
* Humble Logging Library - "THE BEER-WARE LICENSE" (Revision 42) (https://raw.githubusercontent.com/mfreiholz/humblelogging/master/LICENSE)
* ICU - Copyright by International Business Machines Corporation and others: ICU License (http://www.icu-project.org/repos/icu/icu/tags/release-55-1/license.html)
* linenoise - Copyright by Salvatore Sanfilippo (antirez at gmail dot com), Pieter Noordhuis (pcnoordhuis at gmail dot com) : BSD-style license (https://raw.githubusercontent.com/antirez/linenoise/master/LICENSE)
* ncurses - Copyright Free Software Foundation, Inc.: X11 License (http://invisible-island.net/ncurses/ncurses-license.html)
