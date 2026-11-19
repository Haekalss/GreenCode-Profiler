from setuptools import setup
import os
import sys

# Script wrapper yang memanggil bin/greencode.exe atau bin/greencode
setup(
    name="greencode-profiler",
    version="1.7.0",
    description="Universal Process-Level Energy Profiler & Code Quality Analyzer",
    author="GreenCode Team",
    license="MIT",
    packages=["greencode_launcher"],
    entry_points={
        "console_scripts": [
            "greencode=greencode_launcher.cli:main",
        ],
    },
    package_data={
        "greencode_launcher": ["bin/*"],
    },
    include_package_data=True,
    python_requires=">=3.8",
)
