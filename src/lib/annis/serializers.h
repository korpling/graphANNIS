#pragma once

#include <list>

#include <cereal/cereal.hpp>

#include <cereal/types/set.hpp>

#include <google/btree_map.h>
#include <google/btree_set.h>
#include <boost/container/flat_map.hpp>


namespace cereal
{
  namespace  set_detail {


    //! @internal
    template <class Archive, class SetT> inline
    void load_noemplacehint( Archive & ar, SetT & set )
    {
      size_type size;
      ar( make_size_tag( size ) );

      set.clear();

      auto hint = set.begin();
      for( size_type i = 0; i < size; ++i )
      {
        typename SetT::key_type key;

        ar( key );
        hint = set.insert( hint, std::move( key ) );
      }
    }
  }

  /**
   * Save for BTree Maps (which does not have emplace_hint)
   */
  template <class Archive, typename KeyType, typename ValueType>
  void save( Archive & ar, btree::btree_map<KeyType, ValueType> const & map )
  {
    ar( make_size_tag( static_cast<size_type>(map.size()) ) );

    for( const auto & i : map )
      ar( make_map_item(i.first, i.second) );
  }

  /**
   * Load for BTree Maps (which does not have emplace_hint)
   */
  template <class Archive, typename KeyType, typename ValueType>
  void load( Archive & ar, btree::btree_map<KeyType, ValueType> & map )
  {
    size_type size;
    ar( make_size_tag( size ) );

    map.clear();

    auto hint = map.begin();
    for( size_t i = 0; i < size; ++i )
    {
      typename btree::btree_map<KeyType, ValueType>::key_type key;
      typename btree::btree_map<KeyType, ValueType>::mapped_type value;

      ar( make_map_item(key, value) );
      hint = map.insert( hint, std::make_pair(std::move(key), std::move(value)) );
    }
  }

  /**
   * Save for BTree Multimaps (which does not have emplace_hint)
   */
  template <class Archive, typename KeyType, typename ValueType>
  void save( Archive & ar, btree::btree_multimap<KeyType, ValueType> const & map )
  {
    ar( make_size_tag( static_cast<size_type>(map.size()) ) );

    for( const auto & i : map )
      ar( make_map_item(i.first, i.second) );
  }

  /**
   * Load for BTree Multimaps (which does not have emplace_hint)
   */
  template <class Archive, typename KeyType, typename ValueType>
  void load( Archive & ar, btree::btree_multimap<KeyType, ValueType> & map )
  {
    size_type size;
    ar( make_size_tag( size ) );

    map.clear();

    auto hint = map.begin();
    for( size_t i = 0; i < size; ++i )
    {
      typename btree::btree_multimap<KeyType, ValueType>::key_type key;
      typename btree::btree_multimap<KeyType, ValueType>::mapped_type value;

      ar( make_map_item(key, value) );
      hint = map.insert( hint, std::make_pair(std::move(key), std::move(value)) );
    }
  }

  //! Saving for boost::container::flat_set
  template <class Archive, class K, class C, class A> inline
  void CEREAL_SAVE_FUNCTION_NAME( Archive & ar, boost::container::flat_set<K, C, A> const & set )
  {
    set_detail::save( ar, set );
  }

  //! Loading for boost::container::flat_set
  template <class Archive, class K, class C, class A> inline
  void CEREAL_LOAD_FUNCTION_NAME( Archive & ar, boost::container::flat_set<K, C, A> & set )
  {
    set_detail::load( ar, set );
  }

  //! Saving for boost::container::flat_multiset
  template <class Archive, class K, class C, class A> inline
  void CEREAL_SAVE_FUNCTION_NAME( Archive & ar, boost::container::flat_multiset<K, C, A> const & multiset )
  {
    set_detail::save( ar, multiset );
  }

  //! Loading for boost::container::flat_multiset
  template <class Archive, class K, class C, class A> inline
  void CEREAL_LOAD_FUNCTION_NAME( Archive & ar, boost::container::flat_multiset<K, C, A> & multiset )
  {
    set_detail::load( ar, multiset );
  }

  //! Saving for btree::btree_set
  template <class Archive, class K, class C, class A> inline
  void CEREAL_SAVE_FUNCTION_NAME( Archive & ar, btree::btree_set<K, C, A> const & set )
  {
    set_detail::save( ar, set );
  }

  //! Loading for btree::btree_set
  template <class Archive, class K, class C, class A> inline
  void CEREAL_LOAD_FUNCTION_NAME( Archive & ar,btree::btree_set<K, C, A> & set )
  {
    set_detail::load_noemplacehint( ar, set );
  }


  /**
   * Specialized Load Boost Container Flat Map.
   */
  template <class Archive, typename KeyType, typename ValueType>
  void load( Archive & ar, boost::container::flat_map<KeyType, ValueType> & map )
  {
    // This is an adaption of the original load function with a sorted buffer.
    // The stored map is already sorted and unique and  we can use this to save
    // search time when inserting the elments to the flat map.

    using type=typename boost::container::flat_map<KeyType, ValueType>::value_type;

    size_type count;
    ar( make_size_tag( count ) );

    map.clear();

    std::list<std::pair<KeyType, ValueType>> buffer;

    while(count-- > 0){

      KeyType key;
      ValueType value;

      ar( make_map_item(key, value) );

      buffer.push_back({key, value});

      if(buffer.size() >= 1000000 || count == 0)
      {
        map.insert(boost::container::ordered_unique_range, buffer.begin(), buffer.end());
        buffer.clear();
      }
    }

    map.shrink_to_fit();
  }

  /**
   * Specialized Load Boost Container Flat MultiMap.
   */
  template <class Archive, typename KeyType, typename ValueType>
  void load( Archive & ar, boost::container::flat_multimap<KeyType, ValueType> & map )
  {
    // This is an adaption of the original load function with a sorted buffer.
    // The stored multimap is already sorted and  we can use this to save
    // search time when inserting the elments to the flat multimap.

    using type=typename boost::container::flat_multimap<KeyType, ValueType>::value_type;

    size_type count;
    ar( make_size_tag( count ) );

    map.clear();

    std::list<std::pair<KeyType, ValueType>> buffer;

    while(count-- > 0){

      KeyType key;
      ValueType value;

      ar( make_map_item(key, value) );

      buffer.push_back({key, value});

      if(buffer.size() >= 1000000 || count == 0)
      {
        map.insert(boost::container::ordered_range, buffer.begin(), buffer.end());
        buffer.clear();
      }
    }

    map.shrink_to_fit();
  }

} // namespace cereal
