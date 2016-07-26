#pragma once

#include <map>
#include <unordered_map>

#include <boost/container/flat_set.hpp>

namespace annis
{
namespace size_estimation
{
/**
 * Estimate the memory usage of a map in bytes.
 */
template<typename Key, typename Value>
size_t memory(const std::map<Key, Value>& m)
{
  return (sizeof(typename std::map<Key, Value>::value_type) + sizeof(std::_Rb_tree_node_base)) * m.size() + sizeof(m);
}

template<typename Key, typename Value>
size_t memory(const std::unordered_map<Key, Value>& m)
{
  return (m.size() * sizeof(typename std::unordered_map<Key, Value>::value_type)) // actual elements stored
      + (m.bucket_count() * (sizeof(size_t) + sizeof(void*))) // head pointer per bucket
      + (m.size() * sizeof(void*)) // pointer for list entry of each element
      + sizeof(m);
}

template<typename Value>
size_t memory(const boost::container::flat_set<Value>& m)
{
  return (m.size() * sizeof(typename  boost::container::flat_set<Value>::value_type)) // actual elements stored
      + sizeof(m);
}

template<typename Key, typename Value>
size_t memory(const boost::container::flat_map<Key, Value>& m)
{
  return (m.size() * sizeof(typename boost::container::flat_map<Key, Value>::value_type)) // actual elements stored
      + sizeof(m);
}

template<typename Key, typename Value>
size_t memory(const boost::container::flat_multimap<Key, Value>& m)
{
  return (m.size() * sizeof(typename boost::container::flat_multimap<Key, Value>::value_type)) // actual elements stored
      + sizeof(m);
}

}
} // end namespace annis
