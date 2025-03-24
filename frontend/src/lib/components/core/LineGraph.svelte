<script lang="ts">
    import { line, curveLinear, Delaunay, range, scaleLinear, scaleUtc, type ScaleContinuousNumeric, type CurveFactory, type NumberValue } from 'd3';

    interface DataPoint {
        x: number;
        y: number;
    }
    interface DataLine {
        id: string;
        data: DataPoint[];
    }

    interface Props {
        data: DataLine[];

        xLabel?: string;
        xFormat?: string;
        yLabel?: string;
        yFormat?: string;

        curve?: CurveFactory;
        // TODO: These types should be generic (allow Date, etc, as scales can allow for other things)
        xType?: (domain : Iterable<NumberValue>, range: Iterable<number>) => ScaleContinuousNumeric<number, number, never>;
        yType?: (domain : Iterable<NumberValue>, range: Iterable<number>) => ScaleContinuousNumeric<number, number, never>;
    }
    
    let { 
        data,
        xLabel = '',
        xFormat = '', // a format specifier string for the x-axis
        yLabel = '',
        yFormat = '', // a format specifier string for the y-axis

        curve = curveLinear,
        xType = scaleLinear,
        yType = scaleLinear,
     } : Props = $props();

    const dotsFilled = 'white'; // whether dots should be filled or outlined

    const marginTop = 61; // the top margin, in pixels
    const marginRight = 0; // the right margin, in pixels
    const marginBottom = 30; // the bottom margin, in pixels
    const marginLeft = 50; // the left margin, in pixels
    const inset = 0; // inset the default range, in pixels
    const width = 600; // the outer width of the chart, in pixels
    const height = 223; // the outer height of the chart, in pixels
    const horizontalGrid = true; // show horizontal grid lines
    const verticalGrid = true; // show vertical grid lines
    const colors = ['#F50057','#42A5F5','#26A69A','#9575CD']; // fill color for dots && number of colors in fill array MUST match number of subsets in data
    
    const r = 5; // (fixed) radius of dots, in pixels
    const strokeWidth = 5; // stroke width of line, in pixels
    const strokeOpacity = 0.8; // stroke opacity of line
    const tooltipBackground = 'white'; // background color of tooltip
    const tooltipTextColor = 'black'; // text color of tooltip
    const strokeLinecap = 'round'; // stroke line cap of the line
    const strokeLinejoin = 'round'; // stroke line join of the line
    const xScalefactor = width / 80; //y-axis number of values
    const yScalefactor = height / 40; //y-axis number of values
    const insetTop = inset; // inset from top
    const insetRight = inset; // inset from right
    const insetBottom = inset; // inset fro bottom
    const insetLeft = inset; // inset from left
    const xRange = [marginLeft + insetLeft, width - marginRight - insetRight]; // [left, right]
    const yRange = [height - marginBottom - insetBottom, marginTop + insetTop]; // [bottom, top]
  
    let dotInfo : [number[], number, {x: number, y:number}] | null = $state(null);

     let xVals = $derived(data.map((subset) => subset.data.map((el) => el.x)).flat());
     let yVals = $derived(data.map((subset) => subset.data.map((el) => el.y)).flat());
      let colorVals = $derived(data.map((subset, i) => subset.data.map((el) => i)).flat());
      let points = $derived(data.map((subset, i) => subset.data.map(el => ({ x: el.x, y: el.y, color: i }))).flat());
      let subsets = $derived(data.map((subset) => subset.id));

    let I = $derived(range(xVals.length));
    let gaps = (_: { x: number; y: number; color: number; }, i : number) => !isNaN(xVals[i]) && !isNaN(yVals[i]);
    let cleanData = $derived(points.map(gaps));

    let xDomain = $derived([xVals[0], xVals[xVals.length - 1]]);
    let yDomain = $derived([0, Math.max(...yVals)]);

    let xScale = $derived(xType(xDomain, xRange));
    let yScale = $derived(yType(yDomain, yRange));
    let niceY = $derived(scaleLinear().domain([0, Math.max(...yVals)]).nice());

    let lines : string[] = $derived.by(() => {
      let lines : string[] = [];
      colors.forEach((color, j) => {

        let chartLine = line()
          .defined(([i,_]) => cleanData[i])
          .curve(curve)
          .x(([i,_]) => xScale(xVals[i]))
          .y(([_,i]) => yScale(yVals[i]));

      // filter and turn into tuples of i,0
        let filteredI : [number, number][] = I.filter((el, i) => colorVals[i] === j).map((i) => [i, i]);
        let chartedLine = chartLine(filteredI);
        if (chartedLine) lines.push(chartedLine);
      });
      return lines;
    });

    let pointsScaled : number[][] = $derived(points.map((el) => [xScale(el.x), yScale(el.y), el.color]));
    let delaunayGrid = $derived(Delaunay.from(pointsScaled.map((el) => [el[0], el[1]])));
    let voronoiGrid = $derived(delaunayGrid.voronoi([0, 0, width, height]));

    let xTicks = $derived(xScale.ticks(xScalefactor));
    let yTicks = $derived(niceY.ticks(yScalefactor));
  </script>
  <div  style="position:relative;">
  <div class="chart-container" >
    <svg {width} {height} viewBox="0 0 {width} {height}"
      cursor='crosshair'
      on:mouseout="{() => dotInfo = null}"
      on:blur="{() => dotInfo = null}"
    >
      <!-- Dots (if enabled) -->
      {#if !dotInfo}
        {#each I as i}
          <g class='dot' pointer-events='none'>
            <circle
              cx={xScale(xVals[i])}
              cy={yScale(yVals[i])}
              r={r}
              stroke={colors[colorVals[i]]}
              fill={dotsFilled ? colors[colorVals[i]] : 'none'}
            />
          </g>
        {/each}
      {/if}
      <!-- Chart lines -->
      {#each lines as subsetLine, i}
      <g class='chartlines' pointer-events='none'>
        {#if dotInfo}
          <path class="line" fill='none' stroke-opacity={points[dotInfo[1]].color === i ? '1' : '0.1'} stroke={colors[i]} d={subsetLine} stroke-width={strokeWidth} stroke-linecap={strokeLinecap} stroke-linejoin={strokeLinejoin}/>
          <circle cx={xScale(points[dotInfo[1]].x)} cy={yScale(points[dotInfo[1]].y)} r={r} stroke={colors[points[dotInfo[1]].color]} fill={dotsFilled} />
        {:else}
          <path class="line" fill='none' stroke={colors[i]} d={subsetLine}
            stroke-opacity={strokeOpacity} stroke-width={strokeWidth} stroke-linecap={strokeLinecap} stroke-linejoin={strokeLinejoin} />
        {/if}
      </g>
      {/each}
      feasfesfeasfesaf
      <!-- Y-axis and horizontal grid lines -->
      <g class="y-axis" transform="translate({marginLeft}, 0)" pointer-events='none'>
        <path class="domain" stroke="black" d="M{insetLeft}, {marginTop} V{height - marginBottom + 6}"/>
        {#each yTicks as tick, i}
          <g class="tick" transform="translate(0, {yScale(tick)})">
            <line class="tick-start" x1={insetLeft - 6} x2={insetLeft}/>
            {#if horizontalGrid}
              <line class="tick-grid" x1={insetLeft} x2={width - marginLeft - marginRight}/>
            {/if}
            <text  x="-{marginLeft}" y="5">{tick + yFormat}</text>
          </g>
        {/each}
        <text x="-{marginLeft}" y={marginTop - 40}>{yLabel}</text>
      </g>
      <!-- X-axis and vertical grid lines -->
      <g class="x-axis" transform="translate(0,{height - marginBottom - insetBottom})" pointer-events='none'>
        <path class="domain" stroke="black" d="M{marginLeft},0.5 H{width - marginRight}"/>
        {#each xTicks as tick, i}
          <g class="tick" transform="translate({xScale(tick)}, 0)">
            <line class="tick-start" stroke='black' y2='6' />
            {#if verticalGrid}
              <line class="tick-grid" y2={-height + 70} />
            {/if}
            <text font-size='8px' x={-marginLeft/4} y="20">{xTicks[i] + xFormat}</text>
          </g>
        {/each}
        <text x={width - marginLeft - marginRight - 40} y={marginBottom}>{xLabel}</text>
      </g>
      feafseaf
      {#each pointsScaled as point, i}
        <path
          stroke="none"
          fill-opacity="0"
          class="voronoi-cell"
          d={voronoiGrid.renderCell(i)}
          on:mouseover="{(e) => {
            dotInfo = [point, i, {x: e.offsetX, y: e.offsetY}]
          }
          }"
          on:focus="{(e) => {
            // TODO: Something on focus?
            // dotInfo = [point, i, e] 
            }
          }"
        ></path>
      {/each}
    </svg>
  </div>
  <!-- Tooltip -->
  {#if dotInfo}
    <div class="tooltip" style='position:absolute; left:{dotInfo[2].x + 12}px; top:{dotInfo[2].y + 12}px; pointer-events:none; background-color:{tooltipBackground}; color:{tooltipTextColor}'>
      {subsets ? '('+subsets[points[dotInfo[1]].color]+'):' : ''} 
      {points[dotInfo[1]].x}: {points[dotInfo[1]].y.toFixed(2)}{yFormat}
    </div>
  {/if}
</div>
<style>
    .chart-container {
      justify-content: center;
      align-items: center;
      margin-top: 1rem;
      margin-left: 8
      0px;
    }
    svg {
      max-width: 100%;
      height: auto;
      height: "intrinsic";
      margin: auto;
    }
    path {
      fill: "green"
    }
    .y-axis {
      font-size: "10px";
      font-family: sans-serif;
      text-anchor: "end";
    }
    .x-axis {
      font-size: "10px";
      font-family: sans-serif;
      text-anchor: "end";
    }
    .tick {
      opacity: 1;
    }
    .tick-start {
      stroke: black;
      stroke-opacity: 1;
    }
    .tick-grid {
      stroke: black;
      stroke-opacity: 0.2;
      font-size: "11px";
      color: black;
    }
    .tick text {
      fill: black;
      text-anchor: start;
    }
  
    .tooltip{
      border-radius: 5px;
      padding: 5px;
      box-shadow: rgba(0, 0, 0, 0.4) 0px 2px 4px, rgba(0, 0, 0, 0.3) 0px 7px 13px -3px, rgba(0, 0, 0, 0.2) 0px -3px 0px inset;
    }
  </style>