#ifndef TUPEL_H
#define TUPEL_H

#include <cstdint>
#include <string>
#include <cstring>

namespace annis
{
  const std::string annis_ns = "annis4_internal";

  struct Edge
  {
    std::uint32_t source;
    std::uint32_t target;
  };

  enum class ComponentType {COVERAGE, DOMINANCE, POINTING, ORDERING,
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

  /** combines a node ID and the matched annotation */
  typedef std::pair<std::uint32_t, Annotation> Match;

  static Edge constructEdge(std::uint32_t source, std::uint32_t target)
  {
    Edge result;
    result.source = source;
    result.target = target;
    return result;
  }
  
  static Component constructComponent(ComponentType type, const std::string& layer, const std::string& name)
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
}

#endif // TUPEL_H
