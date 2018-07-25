#!/usr/bin/env python3

from setuptools import setup, find_packages

setup(name='graphannis',
      version='0.7.0',
      description='graphANNIS Python API',
      author='Thomas Krause',
      author_email='thomaskrause@posteo.de',
      url='https://github.com/thomaskrause/graphANNIS/',
      packages=['graphannis'],
      include_package_data=True,
      setup_requires=["cffi>=1.0.0"],
      cffi_modules=["package/graphannis_build.py:ffibuilder"],
      install_requires=["cffi>=1.0.0","networkx"],
     )
