#ifndef TUPEL_H
#define TUPEL_H

#include <cstdint>
#include <string>

namespace annis
{
  struct NodeAnnotation
  {
    std::uint32_t nodeId;
    std::string name;
    std::string ns;
    std::string val;
  };

  struct Node
  {
    std::uint32_t id;
    std::string name;
  };
}

#endif // TUPEL_H
