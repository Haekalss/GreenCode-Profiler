from setuptools import setup, find_packages

setup(
    name="greencode-profiler",
    version="1.1.1",
    description="Universal Process-Level Energy Profiler for PHP, Node.js, Python, and Go",
    author="GreenCode Team",
    packages=find_packages(),
    install_requires=[
        "psutil>=5.9.0",
    ],
    entry_points={
        "console_scripts": [
            "greencode=greencode.cli:main",
        ],
    },
    python_requires=">=3.9",
)

