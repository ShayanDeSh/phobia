import pandas as pd
import random

apis = ["/sphinx/v1/predict", "/sphinx/v2/predict", "/sphinx/v3/predict",
        "/yolo/v1/predict", "/yolo/v2/predict", "/yolo/v3/predict"]


def main():
    df = pd.read_excel("./data_6.1~6.30_.xlsx")
    df.loc[df["date"] == 1].sort_values(by=["user id", "start time"])\
      .drop(["date", "month", "location(latitude/lontitude)", "user id"],
            axis=1)\
      .assign(path=lambda x: apis[random.randint(0, len(apis) - 1)])\
      .to_json("./data.json", "records", indent=4)


if __name__ == "__main__":
    main()
