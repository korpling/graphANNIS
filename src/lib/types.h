#ifndef TUPEL_H
#define TUPEL_H

#include <cstdint>
#include <string>

namespace annis
{
  const std::string annis_ns = "annis4_internal";

  typedef std::pair<std::uint32_t, std::uint32_t> Edge;

  enum class ComponentType {COVERAGE, DOMINANCE, POINTING, ORDERING,
                            ComponentType_MAX};

  const size_t MAX_COMPONENT_NAME_SIZE = 255;

  struct Component
  {
    ComponentType type;
    char ns[MAX_COMPONENT_NAME_SIZE];
    char name[MAX_COMPONENT_NAME_SIZE];
  };

  struct Annotation
  {
    std::uint32_t name;
    std::uint32_t ns;
    std::uint32_t val;
  };

  struct Node
  {
    std::uint32_t id;
  };
}

#endif // TUPEL_H
