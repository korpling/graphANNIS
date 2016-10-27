#pragma once

#include <google/btree_map.h>
#include <google/btree_set.h>


#include <boost/container/flat_map.hpp>
#include <boost/container/flat_set.hpp>
#include <boost/container/map.hpp>
#include <boost/container/set.hpp>

#include <boost/serialization/map.hpp>
#include <boost/serialization/set.hpp>
#include <boost/serialization/list.hpp>

namespace boost{
namespace serialization{

////////////////////
/// Google BTree ///
////////////////////

// map (based on STL)

template<class Archive, class Type, class Key, class Compare, class Allocator >
inline void save(Archive & ar, const btree::btree_map<Key, Type, Compare, Allocator> &t, const unsigned int /* file_version */)
{
  boost::serialization::stl::save_collection<Archive, btree::btree_map<Key, Type, Compare, Allocator> >(ar, t);
}
template<class Archive, class Type, class Key, class Compare, class Allocator >
inline void load(Archive & ar, btree::btree_map<Key, Type, Compare, Allocator> &t, const unsigned int /* file_version */){
  load_map_collection(ar, t);
}
template<class Archive, class Type, class Key, class Compare, class Allocator >
inline void serialize(Archive & ar, btree::btree_map<Key, Type, Compare, Allocator> &t, const unsigned int file_version){
  boost::serialization::split_free(ar, t, file_version);
}

// multimap (based on STL)

template<class Archive, class Type, class Key, class Compare, class Allocator >
inline void save(Archive & ar, const btree::btree_multimap<Key, Type, Compare, Allocator> &t, const unsigned int /* file_version */)
{
  boost::serialization::stl::save_collection<Archive, btree::btree_multimap<Key, Type, Compare, Allocator> >(ar, t);
}
template<class Archive, class Type, class Key, class Compare, class Allocator >
inline void load(Archive & ar, btree::btree_multimap<Key, Type, Compare, Allocator> &t, const unsigned int /* file_version */){
  load_map_collection(ar, t);
}
template<class Archive, class Type, class Key, class Compare, class Allocator >
inline void serialize(Archive & ar, btree::btree_multimap<Key, Type, Compare, Allocator> &t, const unsigned int file_version){
  boost::serialization::split_free(ar, t, file_version);
}

//  set (based on STL)
template<class Archive, class Key, class Compare >
inline void save(Archive & ar, const btree::btree_set<Key, Compare> &t, const unsigned int /* file_version */){
  boost::serialization::stl::save_collection<Archive, btree::btree_set<Key, Compare>>(ar, t);
}
template<class Archive, class Key, class Compare >
inline void load(Archive & ar, btree::btree_set<Key, Compare> &t, const unsigned int /* file_version */){
  load_set_collection(ar, t);
}

template<class Archive, class Type, class Key, class Compare >
inline void serialize(Archive & ar, btree::btree_set<Key, Type, Compare> &t, const unsigned int file_version){
  boost::serialization::split_free(ar, t, file_version);
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
inline void load(Archive & ar, boost::container::flat_map<Key, Type, Compare, Allocator> &s, const unsigned int /* file_version */){

  // This is an adaption of the load_map_collection() function with a sorted buffer
  // The stored map is already sorted and unique and we can use this to save
  // search time when inserting the elments to the flat map.

  using type=typename container::flat_map<Key, Type, Compare, Allocator>::value_type;

  s.clear();
  const boost::archive::library_version_type library_version(
      ar.get_library_version()
  );
  // retrieve number of elements
  item_version_type item_version(0);
  collection_size_type count;
  ar >> BOOST_SERIALIZATION_NVP(count);
  if(boost::archive::library_version_type(3) < library_version){
      ar >> BOOST_SERIALIZATION_NVP(item_version);
  }

  std::list<type> buffer;

  while(count-- > 0){

      detail::stack_construct<Archive, type> t(ar, item_version);
      // borland fails silently w/o full namespace
      ar >> boost::serialization::make_nvp("item", t.reference());

      buffer.push_back(t.reference());

      if(buffer.size() >= 1000000 || count == 0)
      {
        s.insert(container::ordered_unique_range, buffer.begin(), buffer.end());
        buffer.clear();
      }
  }

  s.shrink_to_fit();

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
inline void load(Archive & ar, boost::container::flat_multimap<Key, Type, Compare, Allocator> &s, const unsigned int /* file_version */ ){

  // This is an adaption of the load_map_collection() function with a sorted buffer
  // The stored multimap is already sorted and  we can use this to save
  // search time when inserting the elments to the flat multimap.

  using type=typename container::flat_map<Key, Type, Compare, Allocator>::value_type;

  s.clear();
  const boost::archive::library_version_type library_version(
      ar.get_library_version()
  );
  // retrieve number of elements
  item_version_type item_version(0);
  collection_size_type count;
  ar >> BOOST_SERIALIZATION_NVP(count);
  if(boost::archive::library_version_type(3) < library_version){
      ar >> BOOST_SERIALIZATION_NVP(item_version);
  }

  std::list<type> buffer;

  while(count-- > 0){

      detail::stack_construct<Archive, type> t(ar, item_version);
      // borland fails silently w/o full namespace
      ar >> boost::serialization::make_nvp("item", t.reference());

      buffer.push_back(t.reference());

      if(buffer.size() >= 1000000 || count == 0)
      {
        s.insert(container::ordered_range, buffer.begin(), buffer.end());
        buffer.clear();
      }
  }

  s.shrink_to_fit();
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
inline void load(Archive & ar, boost::container::flat_set<Key, Compare, Allocator> &s, const unsigned int /* file_version */){

  // This is an adaption of the load_set_collection() function with a sorted buffer
  // The stored set is already sorted and unique so we can use this to save
  // search time when inserting the elments to the flat multimap.

  using type=typename container::flat_set<Key, Compare, Allocator>::value_type;

  s.clear();
  const boost::archive::library_version_type library_version(
      ar.get_library_version()
  );
  // retrieve number of elements
  item_version_type item_version(0);
  collection_size_type count;
  ar >> BOOST_SERIALIZATION_NVP(count);
  if(boost::archive::library_version_type(3) < library_version){
      ar >> BOOST_SERIALIZATION_NVP(item_version);
  }

  std::list<type> buffer;

  while(count-- > 0){

      detail::stack_construct<Archive, type> t(ar, item_version);
      // borland fails silently w/o full namespace
      ar >> boost::serialization::make_nvp("item", t.reference());

      buffer.push_back(t.reference());

      if(buffer.size() >= 1000000 || count == 0)
      {
        s.insert(container::ordered_unique_range, buffer.begin(), buffer.end());
        buffer.clear();
      }
  }

  s.shrink_to_fit();
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
