import * as React from "react";
import { Map, Source, Layer } from "react-map-gl";
import maplibregl from "maplibre-gl";
import "maplibre-gl/dist/maplibre-gl.css";
import init, {
  calc_weights_from_geojson,
  calc_lisa,
  initSync,
} from "@maticoapp/geostats";

const view ={
            latitude:44.967243, 
            longitude: -102.771556,
            zoom:3
        }

const API_KEY = "32rtUQVfWVrA5316PfSR";
export default function App() {
  const [data, setData] = React.useState(null);
  const [links, setLinks] = React.useState(null);
  const [weightTime, setWeightTime] = React.useState(null);
  const [lisaTime, setLisaTime] = React.useState(null);
  const [weights, setWeights] = React.useState(null);

  const [lisa, setLisa] = React.useState(null);

  React.useEffect(() => {
    init().then(() => {
      fetch(
        "https://raw.githubusercontent.com/Matico-Platform/sample-data/main/sdoh/health-factors-county-chrr.geojson"
      )
        .then((r) => r.json())
        .then((data) => {
          setData(data);
          let weight_start = performance.now()
          let weights = calc_weights_from_geojson(data);
          let weight_end = performance.now()
          setWeightTime(weight_end-weight_start)

          let links = weights.links_geojson(data);
          let values = data.features.map(
            (f) => f.properties["MedianHouseholdIncome"]
          );
          
          let lisa_start=  performance.now()
          let lisa = calc_lisa(weights, values);
          let lisa_end=  performance.now()
          setLisaTime(lisa_end-lisa_start)

          setLisa(lisa);
          setLinks(links);
          setWeights(weights);

          let lisaFeatures= data.features.map((f,index)=>(
            {...f, properties: {...f.properties, pval: lisa.p_vals[index], moran: lisa.moran_val[index], quad: lisa.quads[index] }})
          )
          console.log("lisa features ", lisaFeatures)
          setLisa({type:"FeatureCollection", features: lisaFeatures});
        });
    });
  }, []);

  return (
    <div>
    <div
      style={{
        display:"grid",
        gridTemplateColumns: "1fr 1fr",
        gridTemplateRows: "1fr 1fr",
        gap:"2rem",
        width: "100vw",
        height: "90vh",
      }}
    >
      <Map
        mapLib={maplibregl}
        style={{ width: "100%", height: "100%" }}
        mapStyle={`https://api.maptiler.com/maps/voyager-v2/style.json?key=${API_KEY}`}
        initialViewState={view}
      >
        {data && (
          <Source id="geom-layer" type="geojson" data={data}>
            <Layer
              id="polyLayer"
              type="fill"
              source="geom-layer"
              paint={{"fill-color" : ['interpolate',
                     ['linear'],
                    ['get','MedianHouseholdIncome'],
                    0.00,
                    'rgb(242,240,247)',
                    48000,
                    'rgb(203,201,226)',
                    53000,
                    'rgb(158,154,200)',
                    61000,
                    'rgb(117,107,177)',
                    150000,
                    'rgb(84,39,143)'
              ]}}
            />
          </Source>
        )}
        <h1 style={{position:'relative', zIndex:1000}}>Origional Data</h1>
      </Map>
      <Map
        mapLib={maplibregl}
        style={{ width: "100%", height: "100%" }}
        mapStyle={`https://api.maptiler.com/maps/voyager-v2/style.json?key=${API_KEY}`}
        initialViewState={view}
      >
        {links && (
          <Source id="link-layer" type="geojson" data={links}>
            <Layer
              id="polyLayer"
              type="line"
              source="link-layer"
              layout={{
                "line-join": "round",
                "line-cap": "round",
              }}
              paint={{
                "line-color": "rgba(0, 0, 0, 1.0)",
                "line-width": 0.2,
              }}
            />
          </Source>
        )}
        <h1 style={{position:'relative', zIndex:1000}}>Weights</h1>
      </Map>
      <Map
        mapLib={maplibregl}
        style={{ width: "100%", height: "100%" }}
        mapStyle={`https://api.maptiler.com/maps/voyager-v2/style.json?key=${API_KEY}`}
        initialViewState={view}
      >
        {lisa && (
          <Source id="pval-layer" type="geojson" data={lisa}>
            <Layer
              id="polyLayer"
              type="fill"
              source="pval-layer"
              paint={{"fill-color" : ['interpolate',
                     ['linear'],
                    ['get','pval'],
                    0.00,
                    'rgb(33,102,172)',
                    0.05,
                    'rgb(103,169,207)',
                    0.1,
                    'rgb(209,229,240)',
                    0.15,
                    'rgb(253,219,199)',
                    0.2,
                    'rgb(239,138,98)',
                    1,
                    'rgb(178,24,43)'
              ]
            }}
            />
          </Source>
        )}
        <h1 style={{position:'relative', zIndex:1000}}>P-vals</h1>
      </Map>
      <Map
        mapLib={maplibregl}
        style={{ width: "100%", height: "100%" }}
        mapStyle={`https://api.maptiler.com/maps/voyager-v2/style.json?key=${API_KEY}`}
        initialViewState={view}
      >
        {lisa && (
          <Source id="quad-layer" type="geojson" data={lisa}>
            <Layer
              id="polyLayer"
              type="fill"
              source="quad-layer"
              paint={{
                "fill-color": [
                   'match',
                   ['get','quad'],
                   'HH',
                   '#fe2700',
                   'LL',
                   '#0b32ff',
                   "HL",
                    "#ffa9a7",
                   "LH",
                    "#a6aaff"
                   ,
                  "#888888"
                ],
                "fill-opacity":[
                  'case',
                  ["<=", ['get','pval'],0.05],
                  0.9,
                  0.0
                ]
              }}
            />
          </Source>
        )}
        <h1 style={{position:'relative', zIndex:1000}}>Quads</h1>
      </Map>
    </div>
      {weightTime && <p> Weights took {(weightTime/1000.0).toFixed(2)}s</p>}
      {lisaTime && <p> Lisa took {(lisaTime/1000.0).toFixed(2)}s</p>}
    </div>
  );
}
