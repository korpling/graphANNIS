#pragma once

#include <boost/container/flat_map.hpp>

#include <boost/serialization/map.hpp>
#include <boost/serialization/set.hpp>

namespace boost{
namespace serialization{

// flat map (based on STL)

template<class Archive, class Type, class Key, class Compare, class Allocator >
inline void save(Archive & ar, const boost::container::flat_map<Key, Type, Compare, Allocator> &t, const unsigned int /* file_version */)
{
    boost::serialization::stl::save_collection<Archive,boost::container::flat_map<Key, Type, Compare, Allocator> >(ar, t);
}
template<class Archive, class Type, class Key, class Compare, class Allocator >
inline void load(Archive & ar, boost::container::flat_map<Key, Type, Compare, Allocator> &t, const unsigned int /* file_version */){
    load_map_collection(ar, t);
}
template<class Archive, class Type, class Key, class Compare, class Allocator >
inline void serialize(Archive & ar,boost::container::flat_map<Key, Type, Compare, Allocator> &t, const unsigned int file_version){
    boost::serialization::split_free(ar, t, file_version);
}

// flat multimap (based on STL)

template<class Archive, class Type, class Key, class Compare, class Allocator >
inline void save(Archive & ar, const boost::container::flat_multimap<Key, Type, Compare, Allocator> &t, const unsigned int /* file_version */)
{
    boost::serialization::stl::save_collection<Archive,boost::container::flat_multimap<Key, Type, Compare, Allocator> >(ar, t);
}
template<class Archive, class Type, class Key, class Compare, class Allocator >
inline void load(Archive & ar, boost::container::flat_multimap<Key, Type, Compare, Allocator> &t, const unsigned int /* file_version */){
    load_map_collection(ar, t);
}
template<class Archive, class Type, class Key, class Compare, class Allocator >
inline void serialize(Archive & ar,boost::container::flat_multimap<Key, Type, Compare, Allocator> &t, const unsigned int file_version){
    boost::serialization::split_free(ar, t, file_version);
}

}}
