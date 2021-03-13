import "game" for Game
import "gers.collections" for F64Array, U16Array
import "gers.graphics" for GraphicDevice, VertexArrayObject, VertexArray, Vertex,
  Texture, Shader, Transform2D
import "gers.noise" for Voronoi2D, PoissonDisc

// Lessons Learned
// - Rust needs to send Lists back to Wren
// - F64Array and U16Array need `count` and `toString` properties.
// - VertexArray.add(vertex) is not intuitive. Change to add(x, y, u, v, color)

var Colors = [
  [1.0, 0.0, 0.0, 1.0], // red
  [0.0, 1.0, 0.0, 1.0], // green
  [0.0, 0.0, 1.0, 1.0], // blue
  [1.0, 1.0, 0.0, 1.0], // yellow
  [1.0, 0.0, 1.0, 1.0], // magenta
  [0.0, 1.0, 1.0, 1.0], // cyan
  [1.0, 1.0, 1.0, 1.0], // white
  [0.0, 0.0, 0.0, 1.0], // black
]

var Size = 500
var Scale = 1
var MinPointDistance = 50


class MapGen is Game {
  construct new() {
    super()
  }

  init() {
    __transform = Transform2D.new()
    __transform.setScale(Scale, Scale)

    // var points = createPoints()
    var points = PoissonDisc.new(0, MinPointDistance, Size, Size).generate()
    createDebugPoints(points)

    _voronoi = Voronoi2D.new(points, Size)
    _polygons = []

    // rust-wren can't return lists yet, so we have to copy
    // our polygons out of a foreign class.
    var polygons = _voronoi.makePolygons()

    var polygon = null
    while (polygon = polygons.take()) {
      // System.print("Popped polygon: %(polygon) %(polygon.count)")
      _polygons.add(polygon)
    }

    createVAO()
  }

  draw() {
    var device = GraphicDevice.instance
    device.clearScreen(100, 149, 237, 255)

    device.draw(_vao, _tex, Shader.default, __transform)
    device.draw(_pointsVAO, _pointsTex, Shader.default, __transform)
  }

  createPoints() {
    var points = F64Array.new()
    points.add(30)
    points.add(50)
    
    points.add(60)
    points.add(40)
    
    points.add(60)
    points.add(70)

    points.add(100)
    points.add(150)

    points.add(200)
    points.add(150)

    return points
  }

  createVAO() {
    // Create vertex array object from polygons.
    var vertices = VertexArray.new()
    var indices = U16Array.new()
    var polyOffset = 0
    var polyNum = 0

    for (polygon in _polygons) {
      var i = 0
      var count = polygon.count
      var vertexCount = count / 2
      var vertex = Vertex.new()

      // System.print("Component count: %(count)")
      // System.print("Vertex count: %(vertexCount)")

      if (count >= 3) {
        var color = Colors[polyNum % Colors.count]

        // Don't add polygon if it doesn't at least form a triangle.
        while (i < count) {
          var x = polygon[i]
          var y = polygon[i+1]

          vertex.setPos(x, y)
          vertex.setUv(0, 0)
          vertex.setColor(color[0], color[1], color[2], color[3])
          vertices.add(vertex)

          i = i + 2
        }

        // Indices form a fan, because it's simple
        for (idx in 1...vertexCount-1) {
          indices.add(polyOffset)
          indices.add(polyOffset+idx)
          indices.add(polyOffset+idx+1)
        }

        polyNum = polyNum + 1
        polyOffset = polyOffset + vertexCount
        // System.print("polyOffset %(polyOffset)")
      }
    }

    // System.print("Indices: %(indices.toString())")

    _vao = VertexArrayObject.new(GraphicDevice.instance, vertices, indices)
    _tex = Texture.fromColor(GraphicDevice.instance, 1, 1, 1, 1)
  }

  createDebugPoints(points) {
    System.print("createDebugPoints")

    var vertices = VertexArray.new()
    var indices = U16Array.new()
    var vertex = Vertex.new()
    var size = 5

    for (i in 0...(points.count / 2)) {
      var x = points[2*i]
      var y = points[2*i+1]
      // System.print("Point %(i) (%(x), %(y))")

      vertex.setUv(0, 0)
      vertex.setColor(0.0, 0.0, 1.0, 1.0)

      vertex.setPos(x, y)
      vertices.add(vertex)

      vertex.setPos(x+size, y)
      vertices.add(vertex)

      vertex.setPos(x+size, y+size)
      vertices.add(vertex)

      vertex.setPos(x, y+size)
      vertices.add(vertex)

      indices.add(4*i)
      indices.add(4*i+1)
      indices.add(4*i+2)
      indices.add(4*i)
      indices.add(4*i+2)
      indices.add(4*i+3)
    }

    _pointsVAO = VertexArrayObject.new(GraphicDevice.instance, vertices, indices)
    _pointsTex = Texture.fromColor(GraphicDevice.instance, 1, 1, 1, 1)
  }
}

MapGen.new()
