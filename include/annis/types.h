#pragma once

#include <cstdint>
#include <string>
#include <cstring>
#include <limits>
#include <unordered_map>

// add implemtations for the types defined here to the std::less operator (and some for the std::hash)
#define ANNIS_STRUCT_COMPARE(a, b) {if(a < b) {return true;} else if(a > b) {return false;}}

namespace annis
{
  typedef std::uint32_t nodeid_t;

  const std::string annis_ns = "annis4_internal";
  const std::string annis_node_name = "node_name";
  const std::string annis_tok = "tok";

  const unsigned int uintmax = std::numeric_limits<unsigned int>::max();

  struct Edge
  {
    nodeid_t source;
    nodeid_t target;
  };

  inline bool operator<(const struct Edge &a, const struct Edge &b)
  {
    // compare by source id
    ANNIS_STRUCT_COMPARE(a.source, b.source);

    // if equal compare by target id
    ANNIS_STRUCT_COMPARE(a.target, b.target);

    // they are equal
    return false;
  }

  enum class ComponentType {COVERAGE,
                            INVERSE_COVERAGE,
                            DOMINANCE,
                            POINTING,
                            ORDERING,
                            LEFT_TOKEN,
                            RIGHT_TOKEN,
                            ComponentType_MAX};

  class ComponentTypeHelper
  {
  public:
    static std::string toString(const ComponentType& type)
    {
      switch(type)
      {
      case ComponentType::COVERAGE:
        return "COVERAGE";
        break;
      case ComponentType::INVERSE_COVERAGE:
        return "INVERSE_COVERAGE";
        break;
      case ComponentType::DOMINANCE:
        return "DOMINANCE";
        break;
      case ComponentType::POINTING:
        return "POINTING";
        break;
      case ComponentType::ORDERING:
        return "ORDERING";
        break;
      case ComponentType::LEFT_TOKEN:
        return "LEFT_TOKEN";
        break;
      case ComponentType::RIGHT_TOKEN:
        return "RIGHT_TOKEN";
        break;
      default:
        return "UNKNOWN";
      }
    }
/*
    static ComponentType fromString(const std::string& typeAsString)
    {
      for(unsigned int t = (unsigned int)ComponentType::COVERAGE; t < (unsigned int) ComponentType::ComponentType_MAX; t++)
      {
        if(toString((ComponentType) t) == typeAsString)
        {
          return (ComponentType) t;
        }
      }
      return ComponentType::ComponentType_MAX;
    }
*/
  };

  struct Component
  {
    ComponentType type;
    std::string layer;
    std::string name;
  };
  inline bool operator<(const struct Component &a, const struct Component &b)
  {
    // compare by type
    ANNIS_STRUCT_COMPARE(a.type, b.type);

    // if equal compare by namespace
    ANNIS_STRUCT_COMPARE(a.layer, b.layer);

    // if still equal compare by name
    ANNIS_STRUCT_COMPARE(a.name, b.name);

    // they are equal
    return false;
  }

  struct AnnotationKey
  {
    std::uint32_t name;
    std::uint32_t ns;
  };

  inline bool operator<(const AnnotationKey& a,  const AnnotationKey& b)
  {
    // compare by name (non lexical but just by the ID)
    ANNIS_STRUCT_COMPARE(a.name, b.name);

    // if equal, compare by namespace (non lexical but just by the ID)
    ANNIS_STRUCT_COMPARE(a.ns, b.ns);

    // they are equal
    return false;
  }

  struct Annotation
  {
    std::uint32_t name;
    std::uint32_t ns;
    std::uint32_t val;
  };

  inline bool operator<(const Annotation& a,  const Annotation& b)
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

  inline bool operator==(const Annotation& lhs, const Annotation& rhs)
  {
      return lhs.ns == rhs.ns && lhs.name == rhs.name && lhs.val == rhs.val;
  }

  struct NodeAnnotationKey
  {
    nodeid_t node;
    std::uint32_t anno_name;
    std::uint32_t anno_ns;
  };
  inline bool operator<(const NodeAnnotationKey& a,  const NodeAnnotationKey& b)
  {
    // compare by node ID
    ANNIS_STRUCT_COMPARE(a.node, b.node);

    // compare by name (non lexical but just by the ID)
    ANNIS_STRUCT_COMPARE(a.anno_name, b.anno_name);

    // if equal, compare by namespace (non lexical but just by the ID)
    ANNIS_STRUCT_COMPARE(a.anno_ns, b.anno_ns);

    // they are equal
    return false;
  }

  struct TextProperty
  {
    std::uint32_t textID;
    std::uint32_t val;
  };
  inline bool operator<(const struct TextProperty &a, const struct TextProperty &b)
  {
    ANNIS_STRUCT_COMPARE(a.textID, b.textID);
    ANNIS_STRUCT_COMPARE(a.val, b.val);

    // they are equal
    return false;
  }

  template<typename pos_t>
  struct RelativePosition
  {
    nodeid_t root;
    pos_t pos;
  };


  /** combines a node ID and the matched annotation */
  struct Match
  {
//    bool found;
    nodeid_t node;
    Annotation anno;
  };

  /** Some general statistical numbers specific to a graph component */
  struct GraphStatistic
  {

    /** Flag to indicate whether the statistics was set */
    bool valid;

    bool cyclic;
    bool rootedTree;

    /** number of nodes */
    uint32_t nodes;

    /** Average fan out  */
    double avgFanOut;
    /** maximal number of children of a node */
    uint32_t maxFanOut;
    /** maximum length from a root node to a terminal node */
    uint32_t maxDepth;

    /** only for acyclic graphs: the average number of times a DFS will visit each node */
    double dfsVisitRatio;
  };

  class Init
  {
  public:
    /**
     * @brief initialize an Annotation
     * @param name
     * @param val
     * @param ns
     * @return
     */
    static Annotation initAnnotation(std::uint32_t name = 0, std::uint32_t val=0, std::uint32_t ns=0)
    {
      Annotation result;
      result.name = name;
      result.ns = ns;
      result.val = val;
      return result;
    }

    static Edge initEdge(nodeid_t source, nodeid_t target)
    {
      Edge result;
      result.source = source;
      result.target = target;
      return result;
    }

    static Match initMatch(const Annotation& anno, nodeid_t node)
    {
      Match result;
      result.node = node;
      result.anno = anno;
      return result;
    }
  };

} // end namespace annis





namespace std
{

template <>
struct hash<annis::Annotation>{
public :
  size_t operator()(const annis::Annotation &a ) const{
    return hash<uint32_t>()(a.ns) ^ hash<uint32_t>()(a.name) ^ hash<uint32_t>()(a.val);
  }
};

} // end namespace std

namespace boost
{
namespace serialization
{
template<class Archive>
inline void serialize(
    Archive & ar,
    annis::GraphStatistic & t,
    const unsigned int file_version
    )
{
  ar & t.valid;

  ar & t.cyclic;
  ar & t.rootedTree;

  ar & t.nodes;

  ar & t.avgFanOut;
  ar & t.maxFanOut;
  ar & t.maxDepth;
  ar & t.dfsVisitRatio;
}

template<class Archive>
inline void serialize(
    Archive & ar,
    annis::AnnotationKey & t,
    const unsigned int file_version
    )
{
  ar & t.name;
  ar & t.ns;
}


template<class Archive>
inline void serialize(
    Archive & ar,
    annis::NodeAnnotationKey & t,
    const unsigned int file_version
    )
{
  ar & t.anno_ns;
  ar & t.anno_name;
  ar & t.node;
}

template<class Archive>
inline void serialize(
    Archive & ar,
    annis::Annotation & t,
    const unsigned int file_version
    )
{
  ar & t.ns;
  ar & t.name;
  ar & t.val;
}

template<class Archive>
inline void serialize(
    Archive & ar,
    annis::Edge & t,
    const unsigned int file_version
    )
{
  ar & t.source;
  ar & t.target;
}

template<class Archive, typename T>
inline void serialize(
    Archive & ar,
    annis::RelativePosition<T> & t,
    const unsigned int file_version
    )
{
  ar & t.root;
  ar & t.pos;
}

} // end namespace serialization
} // end namespace boost

