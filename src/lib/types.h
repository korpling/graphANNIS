#ifndef ANNISTYPES_H
#define ANNISTYPES_H

#include <cstdint>
#include <string>
#include <cstring>

namespace annis
{
  typedef std::uint32_t nodeid_t;

  const std::string annis_ns = "annis4_internal";
  const std::string annis_node_name = "node_name";
  const std::string annis_tok = "tok";

  struct Edge
  {
    nodeid_t source;
    nodeid_t target;
  };

  enum class ComponentType {COVERAGE, DOMINANCE, POINTING, ORDERING,
                            LEFT_TOKEN, RIGHT_TOKEN,
                            ComponentType_MAX};
  static std::string ComponentTypeToString(const ComponentType& type)
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

  static ComponentType ComponentTypeFromString(const std::string& typeAsString)
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

  inline bool operator==(const Annotation& lhs, const Annotation& rhs)
  {
      return lhs.ns == rhs.ns && lhs.name == rhs.name && lhs.val == rhs.val;
  }

}

#endif // ANNISTYPES_H
