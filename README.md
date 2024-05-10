# count-me

Sometimes I need to count number of features in GDAL data and keep track of those numbers so I wrote this.

## Objectives

1. Count number of features in a Dataset.
2. Save Counts into csv files if wanted (redirection).
3. Compare counts between Datasets or Counts.

## Usage

Print help:
```sh
countme -h
```

```sh
Count features in GDAL compatible vector data

Usage: countme [SRC]...

Arguments:
  [SRC]...  Path to data sources (see GDAL drivers documentation)

Options:
  -h, --help  Print help (see more with '--help')
```

Then there are two possibilities:

1. Count features in a dataset 

```sh
countme my_dataset
```

where my_dataset is the path to a GDAL supported data (see GDAL drivers documentation).

2. Compare counts between Dataset or previous Counts saved as `csv` files.

```sh
countme my_dataset1 my_count.csv
```

or 

```sh
countme my_count1.csv my_count2.csv
```
