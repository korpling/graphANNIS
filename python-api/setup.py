#!/usr/bin/env python3

from setuptools import setup, find_packages

setup(name='graphannis',
      version='0.7.0',
      description='graphANNIS Python API',
      author='Thomas Krause',
      author_email='thomaskrause@posteo.de',
      url='https://github.com/thomaskrause/graphANNIS/',
      packages=['graphannis'],
      package_data={'graphannis' : ['../target/release/libgraphannis_capi.so']},
      setup_requires=["cffi>=1.0.0"],
      cffi_modules=["package/graphannis_build.py:ffibuilder"],
      install_requires=["cffi>=1.0.0","networkx"],
     )
