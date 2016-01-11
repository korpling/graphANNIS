#ifndef COMPAREFUNCTIONS_H
#define COMPAREFUNCTIONS_H

#include "graphstorage.h"
#include "types.h"
#include <string.h>

#include <tuple>
#include <functional>


namespace annis
{
  
  /**
 * @brief Compares two annotations keys.
 * A value of "0" in any fields of the annnnotation stands for "any value" and always compares to true.
 * @param a
 * @param b
 * @return True if the annotations are the same
 */
inline bool checkAnnotationKeyEqual(const struct Annotation &a, const struct Annotation &b)
{
  // compare by name (non lexical but just by the ID)
  if(a.name != 0 && b.name != 0 &&  a.name != b.name)
  {
    return false;
  }

  // if equal, compare by namespace (non lexical but just by the ID)
  if(a.ns != 0 && b.ns != 0 && a.ns != b.ns)
  {
    return false;
  }

  // they are equal
  return true;
}

/**
 * @brief Compares two annotations.
 * A value of "0" in any fields of the annnnotation stands for "any value" and always compares to true.
 * @param a
 * @param b
 * @return True if the annotations are the same
 */
inline bool checkAnnotationEqual(const struct Annotation &a, const struct Annotation &b)
{
  // compare by name (non lexical but just by the ID)
  if(a.name != 0 && b.name != 0 &&  a.name != b.name)
  {
    return false;
  }

  // if equal, compare by namespace (non lexical but just by the ID)
  if(a.ns != 0 && b.ns != 0 && a.ns != b.ns)
  {
    return false;
  }

  // if still equal compare by value (non lexical but just by the ID)
 if(a.val != 0 && b.val != 0 && a.val != b.val)
  {
    return false;
  }

  // they are equal
  return true;
}

} // end namespace annis



#endif // COMPAREFUNCTIONS_H
