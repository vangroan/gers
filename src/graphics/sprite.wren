
class Sprite {
  x { _x }
  x=(val) { _x = val }

  y { _y }
  y=(val) { _y = val }

  width { _w }
  width=(val) { _w = val }

  height { _h }
  height=(val) { _h = val }

  transform { _transform }
  transform=(val) { _transform = val }
  
  texture { _texture }
  texture=(val) { _texture = val }

  construct new() {
    _x = 0.0
    _y = 0.0
    _w = 1.0
    _h = 1.0
    _transform = Transform2D.new()
    _texture = null
  }
}
