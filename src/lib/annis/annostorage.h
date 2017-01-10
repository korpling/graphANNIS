/* 
 * File:   nodeannostorage.h
 * Author: thomas
 *
 * Created on 14. Januar 2016, 13:53
 */

#pragma once

#include <map>
#include <set>
#include <list>
#include <memory>

#include <google/btree_map.h>
#include <google/btree_set.h>

#include <re2/re2.h>

#include <boost/optional.hpp>
#include <boost/container/flat_map.hpp>
#include <boost/container/flat_set.hpp>
#include <boost/container/map.hpp>
#include <boost/container/set.hpp>

#include <cereal/cereal.hpp>
#include <cereal/types/map.hpp>
#include <cereal/types/set.hpp>
#include <cereal/types/vector.hpp>


#include <annis/types.h>
#include <annis/stringstorage.h>
#include <annis/serializers.h>
#include <annis/util/size_estimator.h>

#include "iterators.h"

namespace annis {

  namespace bc = boost::container;


  template<typename ContainerType,
           class AnnoMap = bc::flat_map<TypeAnnotationKey<ContainerType>, std::uint32_t>,
           class InverseAnnoMap = bc::flat_multimap<Annotation, ContainerType>>
  class AnnoStorage
  {
    friend class DB;
    friend class ExactAnnoValueSearch;
    friend class ExactAnnoKeySearch;
    friend class RegexAnnoSearch;
    friend class NodeByEdgeAnnoSearch;
    friend class AbstractEdgeOperator;

  public:

    using AnnoMap_t = AnnoMap;
    using InverseAnnoMap_t = InverseAnnoMap;

    AnnoStorage() {}
    AnnoStorage(const AnnoStorage& orig) = delete;

    void addAnnotation(ContainerType item, const Annotation& anno)
    {
      annotations.insert(std::pair<TypeAnnotationKey<ContainerType>, uint32_t>({item, anno.name, anno.ns}, anno.val));
      inverseAnnotations.insert(std::pair<Annotation, ContainerType>(anno, item));
      btree::btree_map<AnnotationKey, std::uint64_t>::iterator itKey = annoKeys.find({anno.name, anno.ns});
      if(itKey == annoKeys.end())
      {
         annoKeys.insert({{anno.name, anno.ns}, 1});
      }
      else
      {
         itKey->second++;
      }
    }

    void addAnnotationBulk(std::list<std::pair<TypeAnnotationKey<ContainerType>, ContainerType>> annos)
    {
      annos.sort();
      annotations.insert(annos.begin(), annos.end());

      std::list<std::pair<Annotation, ContainerType>> inverseAnnos;
      std::list<AnnotationKey> annoKeyList;

      for(const auto& entry : annos)
      {
        const TypeAnnotationKey<ContainerType>& key = entry.first;
        inverseAnnos.push_back(std::pair<Annotation, nodeid_t>({key.anno_name, key.anno_ns, entry.second}, key.id));
        annoKeyList.push_back({key.anno_name, key.anno_ns});
      }

      inverseAnnos.sort();

      inverseAnnotations.insert(inverseAnnos.begin(), inverseAnnos.end());

      for(auto annoKey : annoKeyList)
      {
        btree::btree_map<AnnotationKey, size_t>::iterator itKey = annoKeys.find(annoKey);
        if(itKey == annoKeys.end())
        {
           annoKeys.insert({annoKey, 1});
        }
        else
        {
           itKey->second++;
        }
      }

    }

    void deleteAnnotation(ContainerType id, const AnnotationKey& anno)
    {
       auto it = annotations.find({id, anno.name, anno.ns});
       if(it != annotations.end())
       {
          Annotation oldAnno = {anno.name, anno.ns, it->second};
          annotations.erase(it);

          // also delete the inverse annotation
          inverseAnnotations.erase(oldAnno);

          // decrease the annotation count for this key
          btree::btree_map<AnnotationKey, std::uint64_t>::iterator itAnnoKey = annoKeys.find(anno);
          if(itAnnoKey != annoKeys.end())
          {
             itAnnoKey->second--;

             // if there is no such annotation left remove the annotation key from the map
             if(itAnnoKey->second <= 0)
             {
                annoKeys.erase(itAnnoKey);
             }
          }
       }
    }

    inline std::vector<Annotation> getAnnotations(const ContainerType &id, const std::uint32_t& nsID, const std::uint32_t& nameID) const
    {
      auto it = annotations.find({id, nameID, nsID});

      if (it != annotations.end())
      {
        return
        {
          true,
          {
            nameID, nsID, it->second
          }
        };
      }
      return
      {
        false,
        {
          0, 0, 0
        }
      };
    }

    inline std::vector<Annotation> getAnnotations(const StringStorage& strings, const nodeid_t &id, const std::string& ns, const std::string& name) const
    {
      std::pair<bool, std::uint32_t> nsID = strings.findID(ns);
      std::pair<bool, std::uint32_t> nameID = strings.findID(name);

      if (nsID.first && nameID.first)
      {
        return getAnnotations(id, nsID.second, nameID.second);
      }
      return std::vector<Annotation>();
    }

    std::vector<Annotation> getAnnotations(const ContainerType& id) const
    {
      using AnnoIt =  typename AnnoMap_t::const_iterator;

      TypeAnnotationKey<ContainerType> lowerAnno = {id, 0, 0};
      TypeAnnotationKey<ContainerType> upperAnno = {id, uintmax, uintmax};

      std::vector<Annotation> result;
      std::pair<AnnoIt, AnnoIt> itRange = {
        annotations.lower_bound(lowerAnno),
        annotations.upper_bound(upperAnno)
      };

      for (AnnoIt it = itRange.first;
        it != itRange.second; it++)
      {
        const TypeAnnotationKey<ContainerType>& key = it->first;
        result.push_back({key.anno_name, key.anno_ns, it->second});
      }

      return result;
    }

    size_t numberOfAnnotations() const
    {
      return annotations.size();
    }


    void calculateStatistics(const StringStorage& strings)
    {
      const size_t maxHistogramBuckets = 250;
      const size_t maxSampledAnnotations = 2500;

      histogramBounds.clear();

      // collect statistics for each annotation key separatly
      std::map<AnnotationKey, std::vector<std::string>> globalValueList;
      for(const auto& annoKey : annoKeys)
      {
        histogramBounds[annoKey.first] = std::vector<std::string>();
        auto& valueList = globalValueList[annoKey.first] = std::vector<std::string>();

        // get all annotations
        Annotation minAnno = {annoKey.first.name, annoKey.first.ns, 0};
        Annotation maxAnno = {annoKey.first.name, annoKey.first.ns, std::numeric_limits<std::uint32_t>::max()};
        auto itUpperBound = inverseAnnotations.upper_bound(maxAnno);
        std::vector<Annotation> annos;
        for(auto it=inverseAnnotations.lower_bound(minAnno); it != itUpperBound; it++)
        {
          annos.push_back(it->first);
        }
        std::random_shuffle(annos.begin(), annos.end());
        valueList.resize(std::min<size_t>(maxSampledAnnotations, annos.size()));
        for(size_t i=0; i < valueList.size(); i++)
        {
          valueList[i] = strings.str(annos[i].val);
        }

      }

      // create uniformly distributed histogram bounds for each node annotation key
      for(auto it=globalValueList.begin(); it != globalValueList.end(); it++)
      {
        auto& values = it->second;

        std::sort(values.begin(), values.end());

        size_t numValues = values.size();

        size_t numHistBounds = maxHistogramBuckets + 1;
        if(numValues < numHistBounds)
        {
          numHistBounds = numValues;
        }

        if(numHistBounds >= 2)
        {
          auto& h = histogramBounds[it->first];
          h.resize(numHistBounds);

          std::int64_t delta = (numValues-1) / (numHistBounds -1);
          std::int64_t deltaFraction = (numValues -1) % (numHistBounds - 1);

        std::int64_t pos = 0;
        size_t posFraction = 0;
          for(size_t i=0; i < numHistBounds; i++)
          {
            h[i] = values[pos];
            pos += delta;
            posFraction += deltaFraction;

            if(posFraction >= (numHistBounds - 1))
            {
              pos++;
              posFraction -= (numHistBounds - 1);
            }
          }
        }
      }
    }

    bool hasStatistics() const
    {
      return !histogramBounds.empty();
    }

    std::int64_t guessMaxCount(const StringStorage& strings, const Annotation& anno) const
    {
      auto val = strings.strOpt(anno.val);

      if(!val)
      {
        // non existing
        return 0;
      }

      if(anno.ns == 0)
      {
        return guessMaxCount(boost::optional<std::uint32_t>(), anno.name, *val, *val);
      }
      else
      {
        return guessMaxCount(boost::optional<std::uint32_t>(anno.ns), anno.name, *val, *val);
      }
    }
    
    std::int64_t guessMaxCount(const StringStorage& strings, const std::string& ns, const std::string& name, const std::string& val) const
    {
      auto nameID = strings.findID(name);
      if(nameID.first)
      {
        auto nsID = strings.findID(ns);
        if(nsID.first)
        {
          return guessMaxCount(boost::optional<std::uint32_t>(nsID.second), nameID.second,
            val, val);
        }
      }


      // if none of the conditions above is valid the annotation key does not exist
      return 0;
    }

    std::int64_t guessMaxCount(const StringStorage& strings, const std::string& name, const std::string& val) const
    {
      auto nameID = strings.findID(name);
      if(nameID.first)
      {
        return guessMaxCount(boost::optional<std::uint32_t>(), nameID.second, val, val);
      }
      return 0;
    }
    
    std::int64_t guessMaxCountRegex(const StringStorage& strings, const std::string& ns, const std::string& name, const std::string& val) const
    {
      auto nameID = strings.findID(name);
      if(nameID.first)
      {
        auto nsID = strings.findID(ns);
        if(nsID.first)
        {
          re2::RE2 pattern(val);
          if(pattern.ok())
          {
            std::string minMatch;
            std::string maxMatch;
            pattern.PossibleMatchRange(&minMatch, &maxMatch, 10);
            return guessMaxCount(boost::optional<std::uint32_t>(nsID.second), nameID.second, minMatch, maxMatch);
          }
        }
      }

      return 0;
    }

    std::int64_t guessMaxCountRegex(StringStorage& strings, const std::string& name, const std::string& val) const
    {
      auto nameID = strings.findID(name);
      if(nameID.first)
      {
        re2::RE2 pattern(val);
        if(pattern.ok())
        {
          std::string minMatch;
          std::string maxMatch;
          pattern.PossibleMatchRange(&minMatch, &maxMatch, 10);
          return guessMaxCount(boost::optional<std::uint32_t>(), nameID.second, minMatch, maxMatch);
        }
      }
      return 0;
    }

    void clear()
    {
      annotations.clear();
      inverseAnnotations.clear();
      annoKeys.clear();

      histogramBounds.clear();
    }

    void copyStatistics(const btree::btree_map<AnnotationKey, std::vector<std::string>>& stats)
    {
      histogramBounds = stats;
    }

    size_t estimateMemorySize()
    {
      return
          size_estimation::element_size(annotations)
          + size_estimation::element_size(inverseAnnotations)
          + size_estimation::element_size(annoKeys)
          + size_estimation::element_size(histogramBounds);
    }

    virtual ~AnnoStorage() {}

    template <class Archive>
    void serialize( Archive & ar )
    {
      ar(annotations, inverseAnnotations, annoKeys, histogramBounds);
    }

  private:

    /**
     * @brief Maps a fully qualified annotation name for a node to an annotation value
     */
    AnnoMap_t annotations;
    InverseAnnoMap_t inverseAnnotations;

    /// Maps a distinct annotation key to the number of keys available.
    btree::btree_map<AnnotationKey, std::uint64_t> annoKeys;

    /* additional statistical information */
    btree::btree_map<AnnotationKey, std::vector<std::string>> histogramBounds;
    
    
  private:
    /**
     * Internal function for getting an estimation about the number of matches for a certain range of annotation value
     * @param nsID The namespace part of the annotation key. Can be empty (in this case all annotations with the correct name are used).
     * @param nameID The name part of the annotation key.
     * @param lowerVal Inclusive starting point for the value range.
     * @param upperVal Inclusive end point for the value range.
     * @param if true upperVal is inclusive, otherwise it is exclusive
     * @return The estimation of -1 if invalid.
     */
    std::int64_t guessMaxCount(boost::optional<std::uint32_t> nsID, std::uint32_t nameID, const std::string& lowerVal,
      const std::string& upperVal) const
    {
      btree::btree_map<AnnotationKey, std::uint64_t>::const_iterator itBegin;
      btree::btree_map<AnnotationKey, std::uint64_t>::const_iterator itEnd;
      if(nsID)
      {
         itBegin = annoKeys.lower_bound({nameID, *nsID});
         itEnd = annoKeys.upper_bound({nameID, *nsID});
      }
      else
      {
        // find all complete keys which have the given name
         itBegin = annoKeys.lower_bound({nameID, 0});
         itEnd = annoKeys.upper_bound({nameID, std::numeric_limits<std::uint32_t>::max()});
      }

      std::int64_t universeSize = 0;
      std::int64_t sumHistogramBuckets = 0;
      std::int64_t countMatches = 0;
      // guess for each annotation fully qualified key and return the sum of all guesses
      for(auto itKeyCount = itBegin; itKeyCount != itEnd; itKeyCount++)
      {
        universeSize += itKeyCount->second;

        auto itHisto = histogramBounds.find(itKeyCount->first);
        if(itHisto != histogramBounds.end())
        {
          // find the range in which the value is contained
          const auto& histo = itHisto->second;

          // we need to make sure the histogram is not empty -> should have at least two bounds
          if(histo.size() >= 2)
          {
            sumHistogramBuckets += (histo.size() - 1);

            for(size_t i = 0; i < (histo.size()-1); i++)
            {
              const auto& bucketBegin = histo[i];
              const auto& bucketEnd = histo[i+1];
              // check if the range overlaps with the search range
              if(bucketBegin <= upperVal && lowerVal <= bucketEnd)
              {
                countMatches++;
              }
            }
          }
        }
      }

      if(sumHistogramBuckets > 0)
      {
        double selectivity = ((double) countMatches) / ((double) sumHistogramBuckets);
        return std::round(selectivity * ((double) universeSize));
      }
      else
      {
        return 0;
      }
    }
  };

  template<typename ContainerType> using BTreeMultiAnnoStorage =
    AnnoStorage<ContainerType,
      btree::btree_multimap<TypeAnnotationKey<ContainerType>, std::uint32_t>,
      btree::btree_multimap<Annotation, ContainerType>>;
}


