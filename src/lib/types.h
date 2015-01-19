#ifndef ANNISTYPES_H
#define ANNISTYPES_H

#include <cstdint>
#include <string>
#include <cstring>
#include <limits>

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

  enum class ComponentType {COVERAGE, DOMINANCE, POINTING, ORDERING,
                            LEFT_TOKEN, RIGHT_TOKEN,
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
         if(ComponentTypeToString((ComponentType) t) == typeAsString)
         {
           return (ComponentType) t;
         }
       }
       return ComponentType::ComponentType_MAX;
     }
     */
  };




  const size_t MAX_COMPONENT_NAME_SIZE = 255;

  struct Component
  {
    ComponentType type;
    char layer[MAX_COMPONENT_NAME_SIZE];
    char name[MAX_COMPONENT_NAME_SIZE];
  };

  struct Annotation
  {
    std::uint32_t name;
    std::uint32_t ns;
    std::uint32_t val;
  };

  struct TextProperty
  {
    std::uint32_t textID;
    std::uint32_t val;
  };

  struct RelativePosition
  {
    nodeid_t root;
    u_int32_t pos;
  };


  /** combines a node ID and the matched annotation */
  struct Match
  {
//    bool found;
    nodeid_t node;
    Annotation anno;
  };

  /** A combination of two matches together with a flag if a result was found */
  struct BinaryMatch
  {
    bool found;
    Match lhs;
    Match rhs;
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

    static Component initComponent(ComponentType type, const std::string& layer, const std::string& name)
    {
      Component c;
      c.type = type;
      if(layer.size() < MAX_COMPONENT_NAME_SIZE-1 && name.size() < MAX_COMPONENT_NAME_SIZE-1)
      {
        memset(c.layer, 0, MAX_COMPONENT_NAME_SIZE);
        memset(c.name, 0, MAX_COMPONENT_NAME_SIZE);
        layer.copy(c.layer, layer.size());
        if(name != "NULL")
        {
          name.copy(c.name, name.size());
        }
      }
      else
      {
        throw("Component name or namespace are too long");
      }
      return c;
    }

    static RelativePosition initRelativePosition(nodeid_t node, u_int32_t pos)
    {
      RelativePosition result;
      result.root = node;
      result.pos = pos;
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





  inline bool operator==(const Annotation& lhs, const Annotation& rhs)
  {
      return lhs.ns == rhs.ns && lhs.name == rhs.name && lhs.val == rhs.val;
  }

} // end namespace annis

// add implemtations for the types defined here to the std::less operator (and some for the std::hash)
#define ANNIS_STRUCT_COMPARE(a, b) {if(a < b) {return true;} else if(a > b) {return false;}}
namespace std
{

template <>
class hash<annis::Annotation>{
public :
  size_t operator()(const annis::Annotation &a ) const{
    return hash<uint32_t>()(a.ns) ^ hash<uint32_t>()(a.name) ^ hash<uint32_t>()(a.val);
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

#endif // ANNISTYPES_H
