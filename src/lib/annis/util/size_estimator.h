#pragma once

#include <map>
#include <unordered_map>

#include <boost/container/flat_map.hpp>
#include <boost/container/flat_set.hpp>

#include <google/btree_map.h>
#include <google/btree_set.h>

namespace annis
{
/**
 * Includes functions to estimate the used main memory of some containers in bytes.
 */
namespace size_estimation
{

template<typename Key, typename Value>
size_t element_size(const std::unordered_map<Key, Value>& m)
{
  return (m.size() * sizeof(typename std::unordered_map<Key, Value>::value_type)) // actual elements stored
      + (m.bucket_count() * (sizeof(size_t) + sizeof(void*))) // head pointer per bucket
      + (m.size() * sizeof(void*)); // pointer for list entry of each element
;
}

template<typename Key>
size_t element_size(const boost::container::flat_set<Key>& m)
{
  return (m.size() * sizeof(typename  boost::container::flat_set<Key>::value_type)); // actual elements stored;
}

template<typename Key, typename Value>
size_t element_size(const boost::container::flat_map<Key, Value>& m)
{
  return (m.size() * sizeof(typename boost::container::flat_map<Key, Value>::value_type)); // actual elements stored;
}

template<typename Key, typename Value>
size_t element_size(const boost::container::flat_multimap<Key, Value>& m)
{
  return (m.size() * sizeof(typename boost::container::flat_multimap<Key, Value>::value_type)); // actual elements stored;
}

template<typename Key, typename Value>
size_t element_size(const btree::btree_map<Key, Value>& m)
{
  return m.bytes_used();
}

template<typename Key, typename Value>
size_t element_size(const btree::btree_multimap<Key, Value>& m)
{
  return m.bytes_used();
}

template<typename Key>
size_t element_size(const  btree::btree_set<Key>& m)
{
  return m.bytes_used();
}

}
} // end namespace annis
