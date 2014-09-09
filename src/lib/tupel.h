#ifndef TUPEL_H
#define TUPEL_H

#include <cstdint>
#include <string>

namespace annis
{
  struct Annotation
  {
    std::string name;
    std::string ns;
    std::string val;
  };

  struct Node
  {
    std::uint32_t id;
    std::string name;
  };

  struct Edge
  {
    std::uint32_t source;
    std::uint32_t target;
    std::uint32_t component;
  };
}

#endif // TUPEL_H
