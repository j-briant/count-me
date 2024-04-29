# count-me

Sometimes I need to count number of features in GDAL data and keep track of those numbers so I wrote this.

## Objectives

1. Count number of features in a Dataset.
2. Save Counts into csv files if wanted.
3. Compare counts between Datasets or Counts.

## Usage

Two possibilities:

1. Count features in a dataset 

```sh
count-me my_dataset
```

where my_dataset is the path to a GDAL supported data (see GDAL drivers documentation).

2. Compare counts between Dataset or Counts.

```sh
count-me my_dataset1 my_count.csv
```

or 

```sh
count-me my_count1.csv my_count2.csv
```

## Why Rust?

Because why not?