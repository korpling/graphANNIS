#pragma once

#include <stx/btree_map>
#include <stx/btree_multimap>
#include <stx/btree_set>

#include <boost/container/flat_map.hpp>
#include <boost/container/flat_set.hpp>
#include <boost/container/map.hpp>
#include <boost/container/set.hpp>

#include <boost/serialization/map.hpp>
#include <boost/serialization/set.hpp>
#include <boost/serialization/list.hpp>

namespace boost{
namespace serialization{

///////////
/// STX ///
///////////

// map (based on STL)

template<class Archive, class Type, class Key, class Compare, class Allocator >
inline void save(Archive & ar, const stx::btree_map<Key, Type, Compare, Allocator> &t, const unsigned int /* file_version */)
{
  boost::serialization::stl::save_collection<Archive, stx::btree_map<Key, Type, Compare, Allocator> >(ar, t);
}
template<class Archive, class Type, class Key, class Compare, class Allocator >
inline void load(Archive & ar, stx::btree_map<Key, Type, Compare, Allocator> &t, const unsigned int /* file_version */){
  load_map_collection(ar, t);
}
template<class Archive, class Type, class Key, class Compare, class Allocator >
inline void serialize(Archive & ar, stx::btree_map<Key, Type, Compare, Allocator> &t, const unsigned int file_version){
  boost::serialization::split_free(ar, t, file_version);
}

// multimap (based on STL)

template<class Archive, class Type, class Key, class Compare, class Allocator >
inline void save(Archive & ar, const stx::btree_multimap<Key, Type, Compare, Allocator> &t, const unsigned int /* file_version */)
{
  boost::serialization::stl::save_collection<Archive, stx::btree_multimap<Key, Type, Compare, Allocator> >(ar, t);
}
template<class Archive, class Type, class Key, class Compare, class Allocator >
inline void load(Archive & ar, stx::btree_multimap<Key, Type, Compare, Allocator> &t, const unsigned int /* file_version */){
  load_map_collection(ar, t);
}
template<class Archive, class Type, class Key, class Compare, class Allocator >
inline void serialize(Archive & ar, stx::btree_multimap<Key, Type, Compare, Allocator> &t, const unsigned int file_version){
  boost::serialization::split_free(ar, t, file_version);
}

//  set (based on STL)
template<class Archive, class Key, class Compare, class Allocator >
inline void save(Archive & ar, const stx::btree_set<Key, Compare, Allocator> &t, const unsigned int /* file_version */){
  boost::serialization::stl::save_collection<Archive, stx::btree_set<Key, Compare, Allocator>>(ar, t);
}
template<class Archive, class Key, class Compare, class Allocator >
inline void load(Archive & ar, stx::btree_set<Key, Compare, Allocator> &t, const unsigned int /* file_version */){
  load_set_collection(ar, t);
}

/////////////
/// Boost ///
/////////////

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
inline void load(Archive & ar, boost::container::flat_multimap<Key, Type, Compare, Allocator> &t, const unsigned int /* file_version */ ){
  load_map_collection(ar, t);
}
template<class Archive, class Type, class Key, class Compare, class Allocator >
inline void serialize(Archive & ar,boost::container::flat_multimap<Key, Type, Compare, Allocator> &t, const unsigned int file_version){
  boost::serialization::split_free(ar, t, file_version);
}

// flat set (based on STL)
template<class Archive, class Key, class Compare, class Allocator >
inline void save(Archive & ar, const boost::container::flat_set<Key, Compare, Allocator> &t, const unsigned int /* file_version */){
  boost::serialization::stl::save_collection<Archive, boost::container::flat_set<Key, Compare, Allocator>>(ar, t);
}
template<class Archive, class Key, class Compare, class Allocator >
inline void load(Archive & ar, boost::container::flat_set<Key, Compare, Allocator> &t, const unsigned int /* file_version */){
  load_set_collection(ar, t);
}
// split non-intrusive serialization function member into separate
// non intrusive save/load member functions
template<class Archive, class Key, class Compare, class Allocator >
inline void serialize( Archive & ar, boost::container::flat_set<Key, Compare, Allocator> & t,const unsigned int file_version){
  boost::serialization::split_free(ar, t, file_version);
}

// boost map (based on STL)

template<class Archive, class Type, class Key, class Compare, class Allocator, class MapOptions >
inline void save(Archive & ar, const boost::container::map<Key, Type, Compare, Allocator, MapOptions> &t, const unsigned int /* file_version */)
{
  boost::serialization::stl::save_collection<Archive,boost::container::map<Key, Type, Compare, Allocator, MapOptions> >(ar, t);
}
template<class Archive, class Type, class Key, class Compare, class Allocator, class MapOptions >
inline void load(Archive & ar, boost::container::map<Key, Type, Compare, Allocator, MapOptions> &t, const unsigned int /* file_version */){
  load_map_collection(ar, t);
}
template<class Archive, class Type, class Key, class Compare, class Allocator, class MapOptions >
inline void serialize(Archive & ar,boost::container::map<Key, Type, Compare, Allocator, MapOptions> &t, const unsigned int file_version){
  boost::serialization::split_free(ar, t, file_version);
}

// boost multimap (based on STL)

template<class Archive, class Type, class Key, class Compare, class Allocator, class MapOptions >
inline void save(Archive & ar, const boost::container::multimap<Key, Type, Compare, Allocator, MapOptions> &t, const unsigned int /* file_version */)
{
  boost::serialization::stl::save_collection<Archive,boost::container::multimap<Key, Type, Compare, Allocator, MapOptions> >(ar, t);
}
template<class Archive, class Type, class Key, class Compare, class Allocator, class MapOptions >
inline void load(Archive & ar, boost::container::multimap<Key, Type, Compare, Allocator, MapOptions> &t, const unsigned int /* file_version */){
  load_map_collection(ar, t);
}
template<class Archive, class Type, class Key, class Compare, class Allocator, class MapOptions >
inline void serialize(Archive & ar,boost::container::multimap<Key, Type, Compare, Allocator, MapOptions> &t, const unsigned int file_version){
  boost::serialization::split_free(ar, t, file_version);
}

}}
