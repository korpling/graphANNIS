#ifndef COMPAREFUNCTIONS_H
#define COMPAREFUNCTIONS_H

#include "edgedb.h"
#include "types.h"
#include <string.h>

#include <tuple>
#include <functional>

namespace annis
{

#define ANNIS_STRUCT_COMPARE(a, b) {if(a < b) {return true;} else if(a > b) {return false;}}

struct compComponent
{
  bool operator()(const struct Component &a, const struct Component &b) const
  {
    // compare by type
    ANNIS_STRUCT_COMPARE(a.type, b.type);

    // if equal compare by namespace
    int nsCompare = strncmp(a.layer, b.layer, MAX_COMPONENT_NAME_SIZE);
    ANNIS_STRUCT_COMPARE(nsCompare, 0);

    // if still equal compare by name
    int nameCompare = strncmp(a.name, b.name, MAX_COMPONENT_NAME_SIZE);
    ANNIS_STRUCT_COMPARE(nameCompare, 0);

    // they are equal
    return false;
  }
};

struct compAnno
{
  bool operator()(const struct Annotation &a, const struct Annotation &b) const
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

struct compEdges
{
  bool operator()(const struct Edge &a, const struct Edge &b) const
  {
    // compare by source id
    ANNIS_STRUCT_COMPARE(a.source, b.source);

    // if equal compare by target id
    ANNIS_STRUCT_COMPARE(a.target, b.target);

    // they are equal
    return false;
  }
};

struct compTextProperty
{
  bool operator()(const struct TextProperty &a, const struct TextProperty &b) const
  {
    ANNIS_STRUCT_COMPARE(a.textID, b.textID);
    ANNIS_STRUCT_COMPARE(a.val, b.val);

    // they are equal
    return false;
  }
};

struct compRelativePosition
{
  bool operator()(const struct RelativePosition &a, const struct RelativePosition &b) const
  {
    ANNIS_STRUCT_COMPARE(a.root, b.root);
    ANNIS_STRUCT_COMPARE(a.pos, b.pos);

    // they are equal
    return false;
  }
};

struct compMatch
{
  bool operator()(const struct Match &a, const struct Match &b) const
  {
    ANNIS_STRUCT_COMPARE(a.node, b.node);
    ANNIS_STRUCT_COMPARE(a.anno.name, b.anno.name);
    ANNIS_STRUCT_COMPARE(a.anno.ns, b.anno.ns);
    ANNIS_STRUCT_COMPARE(a.anno.val, b.anno.val);
    return false;
  }
};

struct compBinaryMatch
{
  bool operator()(const struct BinaryMatch &a, const struct BinaryMatch &b) const
  {

    ANNIS_STRUCT_COMPARE(a.lhs.node, b.lhs.node);
    ANNIS_STRUCT_COMPARE(a.lhs.anno.name, b.lhs.anno.name);
    ANNIS_STRUCT_COMPARE(a.lhs.anno.ns, b.lhs.anno.ns);
    ANNIS_STRUCT_COMPARE(a.lhs.anno.val, b.lhs.anno.val);

    ANNIS_STRUCT_COMPARE(a.rhs.node, b.rhs.node);
    ANNIS_STRUCT_COMPARE(a.rhs.anno.name, b.rhs.anno.name);
    ANNIS_STRUCT_COMPARE(a.rhs.anno.ns, b.rhs.anno.ns);
    ANNIS_STRUCT_COMPARE(a.rhs.anno.val, b.rhs.anno.val);

    return false;
  }
};



} // end namespace annis

namespace std {
    template <>
        class hash<annis::Annotation>{
        public :
        size_t operator()(const annis::Annotation &a ) const{
            return hash<uint32_t>()(a.ns) ^ hash<uint32_t>()(a.name) ^ hash<uint32_t>()(a.val);
        }
    };
}

#endif // COMPAREFUNCTIONS_H
