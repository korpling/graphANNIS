#include "edgedb.h"

#include <boost/archive/binary_oarchive.hpp>
#include <boost/archive/binary_iarchive.hpp>

#include <fstream>

using namespace annis;
using namespace std;


bool ReadableGraphStorage::load(string dirPath)
{
  statistics.valid = false;
  ifstream in;

  in.open(dirPath + "/statistics.archive");
  if(in.is_open())
  {
    boost::archive::binary_iarchive ia(in);
    ia >> statistics;
    in.close();
  }
  return true;
}

bool ReadableGraphStorage::save(string dirPath)
{
  ofstream out;

  out.open(dirPath + "/statistics.archive");
  boost::archive::binary_oarchive oa(out);
  oa << statistics;
  out.close();

  return true;
}
