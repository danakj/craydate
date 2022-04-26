/// A three-dimensional vector.
#[derive(Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Vector3<T> {
  /// The x component.
  pub x: T,
  /// The y component.
  pub y: T,
  /// The z component.
  pub z: T,
}
