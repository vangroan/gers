
# Develop

Start by creating a Python 3.7 virtual environment.

On Windows, use `py`:

```
py -3.7 -m virtualenv venv 
venv/Scripts/activate.bat
```

On Linux and macOS:

```
$ python3 -m virtualenv venv
$ source venv/bin/activate
```

Install the python wrapper project into the virtual environment.

```
cd python
pip install -e .
```
