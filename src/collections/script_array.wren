
foreign class U16Array {
  construct new() {}

  foreign get(index)
  foreign add(value)
  foreign insert(index, value)
  foreign removeAt(index)
  foreign clear()
  // TODO: count
  // TODO: toString
  foreign iterate(iterator)
  foreign iteratorValue(iterator)
}
