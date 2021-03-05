
/**
 * There is a known edge case where points with the same
 * x-coordinate or y-coordinate causes degenerate polygons.
 *
 * A workaround is implemented where conflicting points
 * are shifted until all conflicts are resolved.
 */
class Voronoi2D {
  /* 2D points, packed into a flat array. */
  points { _points }

  /**
   * Bounding box size, both width and height, that will
   * contain the generated polygons.
   */
  boxSize { _size }

  /**
   * Creates a 2-dimensional Voronoi generator which uses
   * Fortune's algorithm.
   *
   * # Errors
   *
   * Aborts the fiber if the number of elements in the
   * points array is not even.
   *
   * @param points    gers.collections.F64Array 2D points packed into float array.
   * @param boxSize   f64                       Square bounding box size.
   */
  construct new(points, boxSize) {
    _points = points
    _size = boxSize
  }

  foreign makePolygons()
}
