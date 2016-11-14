#pragma once

#include <cstdint>
#include <string>
#include <cstring>
#include <limits>
#include <unordered_map>

#include <tuple>

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

  template<class Archive>
  void serialize(Archive & archive,
                 Edge & m)
  {
    archive( m.source, m.target);
  }

  inline bool operator<(const struct Edge &a, const struct Edge &b)
  {
    return std::tie(a.source, a.target) < std::tie(b.source, b.target);
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

  };

  struct Component
  {
    ComponentType type;
    std::string layer;
    std::string name;
  };
  inline bool operator<(const struct Component &a, const struct Component &b)
  {
    return std::tie(a.type, a.layer, a.name) < std::tie(b.type, b.layer, b.name);
  }

  struct AnnotationKey
  {
    std::uint32_t name;
    std::uint32_t ns;
  };

  template<class Archive>
  void serialize(Archive & archive,
                 AnnotationKey & m)
  {
    archive(m.name, m.ns );
  }


  inline bool operator<(const AnnotationKey& a,  const AnnotationKey& b)
  {
    return std::tie(a.name, a.ns) < std::tie(b.name, b.ns);
  }

  struct Annotation
  {
    std::uint32_t name;
    std::uint32_t ns;
    std::uint32_t val;
  };

  template<class Archive>
  void serialize(Archive & archive,
                 Annotation & m)
  {
    archive(m.name, m.ns, m.val);
  }

  inline bool operator<(const Annotation& a,  const Annotation& b)
  {
    return std::tie(a.name, a.ns, a.val) < std::tie(b.name, b.ns, b.val);
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
    return std::tie(a.node, a.anno_name, a.anno_ns) < std::tie(b.node, b.anno_name, b.anno_ns);
  }

  template<class Archive>
  void serialize(Archive & archive,
                 NodeAnnotationKey & m)
  {
    archive(m.node, m.anno_name, m.anno_ns);
  }

  struct TextProperty
  {
    std::uint32_t corpusID;
    std::uint32_t textID;
    std::uint32_t val;
  };
  inline bool operator<(const struct TextProperty &a, const struct TextProperty &b)
  {
    return std::tie(a.corpusID, a.textID, a.val) < std::tie(b.corpusID, b.textID, b.val);
  }

  template<typename pos_t>
  struct RelativePosition
  {
    nodeid_t root;
    pos_t pos;
  };

  template<class Archive, typename pos_t>
  void serialize(Archive & archive,
                 RelativePosition<pos_t> & m)
  {
    archive(m.root, m.pos );
  }



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

  template<class Archive>
  void serialize(Archive & archive,
                 GraphStatistic & m)
  {
    archive(m.valid, m.cyclic, m.rootedTree, m.nodes, m.avgFanOut, m.maxFanOut, m.maxDepth, m.dfsVisitRatio);
  }

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

