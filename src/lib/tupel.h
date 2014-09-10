#ifndef TUPEL_H
#define TUPEL_H

#include <cstdint>
#include <string>

namespace annis
{
  const std::string annis_ns = "annis4_internal";

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

  struct Edge
  {
    std::uint32_t source;
    std::uint32_t target;
    std::uint32_t component;
  };
}

#endif // TUPEL_H
