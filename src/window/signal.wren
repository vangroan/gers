
/**
 * Event signalling for implementing the observer
 * pattern.
 */
class Signal {
  construct new() {
    _subs = []
  }

  /**
   * Subscribes the given function to receive
   * emitted signals.
   *
   * The signal will keep the function from
   * being garbage collected. To release the
   * function, call `Signal.remove(fn)`.
   */
  add(fn) {
    _subs.add(fn)
  }

  /**
   * Removes the given function from the signal.
   *
   * Returns `true` if the given function was
   * removed.
   */
  remove(fn) {
    var index = _subs.indexOf(fn)
    
    if (index < 0) {
      return false
    }

    return _subs.removeAt(index) != null
  }

  /* Returns true when no subscribers have been added. */
  isEmpty { _subs.count == 0 }

  send() {
    for (sub in _subs) {
      sub.call()
    }
  }

  send(a) {
    for (sub in _subs) {
      sub.call(a)
    }
  }

  send(a, b) {
    for (sub in _subs) {
      sub.call(a, b)
    }
  }

  send(a, b, c) {
    for (sub in _subs) {
      sub.call(a, b, c)
    }
  }

  send(a, b, c, d) {
    for (sub in _subs) {
      sub.call(a, b, c, d)
    }
  }
}
