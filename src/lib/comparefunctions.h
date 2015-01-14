#ifndef COMPAREFUNCTIONS_H
#define COMPAREFUNCTIONS_H

#include "edgedb.h"
#include "types.h"
#include <string.h>

#include <tuple>
#include <functional>


#define ANNIS_STRUCT_COMPARE(a, b) {if(a < b) {return true;} else if(a > b) {return false;}}


namespace std {
template <>
class hash<annis::Annotation>{
public :
  size_t operator()(const annis::Annotation &a ) const{
    return hash<uint32_t>()(a.ns) ^ hash<uint32_t>()(a.name) ^ hash<uint32_t>()(a.val);
  }
};

template<>
struct less<annis::Annotation>
{
  bool operator()(const annis::Annotation& a,  const annis::Annotation& b) const
  {
    // compare by name (non lexical but just by the ID)
    ANNIS_STRUCT_COMPARE(a.name, b.name);

    // if equal, compare by namespace (non lexical but just by the ID)
    ANNIS_STRUCT_COMPARE(a.ns, b.ns);

    // if still equal compare by value (non lexical but just by the ID)
    ANNIS_STRUCT_COMPARE(a.val, b.val);

    // they are equal
    return false;
  }
};

template<>
struct less<annis::Component>
{
  bool operator()(const struct annis::Component &a, const struct annis::Component &b) const
  {
    // compare by type
    ANNIS_STRUCT_COMPARE(a.type, b.type);

    // if equal compare by namespace
    int nsCompare = strncmp(a.layer, b.layer, annis::MAX_COMPONENT_NAME_SIZE);
    ANNIS_STRUCT_COMPARE(nsCompare, 0);

    // if still equal compare by name
    int nameCompare = strncmp(a.name, b.name, annis::MAX_COMPONENT_NAME_SIZE);
    ANNIS_STRUCT_COMPARE(nameCompare, 0);

    // they are equal
    return false;
  }
};

template<>
struct less<annis::Edge>
{
  bool operator()(const struct annis::Edge &a, const struct annis::Edge &b) const
  {
    // compare by source id
    ANNIS_STRUCT_COMPARE(a.source, b.source);

    // if equal compare by target id
    ANNIS_STRUCT_COMPARE(a.target, b.target);

    // they are equal
    return false;
  }
};

template<>
struct less<annis::TextProperty>
{
  bool operator()(const struct annis::TextProperty &a, const struct annis::TextProperty &b) const
  {
    ANNIS_STRUCT_COMPARE(a.textID, b.textID);
    ANNIS_STRUCT_COMPARE(a.val, b.val);

    // they are equal
    return false;
  }
};
} // end namespace std

namespace annis
{

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
