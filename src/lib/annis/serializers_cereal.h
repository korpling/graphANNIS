#pragma once

#include <cereal/cereal.hpp>

#include <cereal/types/set.hpp>

#include <google/btree_map.h>
#include <boost/container/flat_map.hpp>



namespace cereal
{
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

} // namespace cereal
