// const WS_URL='ws://localhost:8080/ws';const MAX_POINTS=60;
// const data={temp:[],humidity:[],sound:[],hr:[]};
// let ws;const charts={};
// function connect(){
//   const status=document.getElementById('connectionStatus');
//   status.textContent='Connecting...';status.className='connection-status';
//   ws=new WebSocket(WS_URL);
//   ws.onopen=()=>{status.textContent='Connected';status.className='connection-status connected'};
//   ws.onclose=()=>{status.textContent='Disconnected';status.className='connection-status error';setTimeout(connect,3000)};
//   ws.onerror=()=>{status.textContent='Error';status.className='connection-status error'};
//   ws.onmessage=(e)=>{
//     try{const msg=JSON.parse(e.data);if(msg.type==='SensorUpdate')updateData(msg.data)}catch(err){console.error(err)}
//   };
// }
// function updateData(r){
//   const t=new Date(r.timestamp);
//   [{arr:data.temp,val:r.temperature,el:'tempValue',chart:'tempChart'},
//    {arr:data.humidity,val:r.humidity,el:'humidityValue',chart:'humidityChart'},
//    {arr:data.sound,val:r.sound,el:'soundValue',chart:'soundChart'},
//    {arr:data.hr,val:r.heart_rate,el:'hrValue',chart:'hrChart'}
//   ].forEach(({arr,val,el,chart})=>{
//     arr.push({time:t,value:val});if(arr.length>MAX_POINTS)arr.shift();
//     document.getElementById(el).textContent=val.toFixed(1);
//     updateChart(chart,arr);
//   });
//   updateIndicators(r);
// }
// function updateChart(id,arr){
//   const c=document.getElementById(id);if(!arr.length)return;
//   const w=c.clientWidth,h=c.clientHeight,m={t:10,r:10,b:20,l:40};
//   const x=d3.scaleTime().domain(d3.extent(arr,d=>d.time)).range([m.l,w-m.r]);
//   const y=d3.scaleLinear().domain([d3.min(arr,d=>d.value)*0.95,d3.max(arr,d=>d.value)*1.05]).range([h-m.b,m.t]);
//   const line=d3.line().x(d=>x(d.time)).y(d=>y(d.value)).curve(d3.curveMonotoneX);
//   const area=d3.area().x(d=>x(d.time)).y0(h-m.b).y1(d=>y(d.value)).curve(d3.curveMonotoneX);
//   d3.select(c).selectAll('*').remove();
//   const svg=d3.select(c).append('svg').attr('viewBox',`0 0 ${w} ${h}`);
//   const grad=svg.append('defs').append('linearGradient').attr('id',id+'Grad').attr('x1','0%').attr('y1','0%').attr('x2','0%').attr('y2','100%');
//   grad.append('stop').attr('offset','0%').attr('stop-color','#00d4aa').attr('stop-opacity',0.3);
//   grad.append('stop').attr('offset','100%').attr('stop-color','#00d4aa').attr('stop-opacity',0);
//   svg.append('path').datum(arr).attr('fill',`url(#${id}Grad)`).attr('d',area);
//   svg.append('path').datum(arr).attr('fill','none').attr('stroke','#00d4aa').attr('stroke-width',2).attr('d',line);
// }
// function updateIndicators(r){
//   const el=document.getElementById('stressIndicators');el.innerHTML='';
//   const ind=[];
//   if(r.temperature>28)ind.push({t:'High Temperature',h:true});
//   if(r.temperature<18)ind.push({t:'Low Temperature',h:false});
//   if(r.humidity>70)ind.push({t:'High Humidity',h:false});
//   if(r.humidity<30)ind.push({t:'Low Humidity',h:false});
//   if(r.sound>500)ind.push({t:'High Noise',h:true});
//   if(r.heart_rate>100)ind.push({t:'Elevated HR',h:true});
//   if(r.heart_rate<50)ind.push({t:'Low HR',h:false});
//   ind.forEach(i=>{const d=document.createElement('div');d.className='stress-indicator'+(i.h?' high':'');d.textContent=i.t;el.appendChild(d)});
//   if(!ind.length){const d=document.createElement('div');d.className='stress-indicator';d.style.background='rgba(0,212,170,0.15)';d.style.color='#00d4aa';d.textContent='All metrics normal';el.appendChild(d)}
// }
// connect();


const WS_URL = 'ws://localhost:8080/ws';
const MAX_POINTS = 60;

const data = { temp:[], humidity:[], sound:[], hr:[] };
let ws;

function connect(){
  const status = document.getElementById('connectionStatus');
  ws = new WebSocket(WS_URL);

  ws.onopen = () => {
    status.textContent = 'Connected';
    status.className = 'connection-status connected';
  };

  ws.onclose = () => {
    status.textContent = 'Disconnected';
    status.className = 'connection-status error';
    setTimeout(connect,3000);
  };

  ws.onerror = () => {
    status.textContent = 'Error';
    status.className = 'connection-status error';
  };

  ws.onmessage = e => {
    const msg = JSON.parse(e.data);
    if(msg.type === 'SensorUpdate'){
      updateData(msg.data);
    }
  };
}

function updateData(r){
  const t = new Date(r.timestamp);

  [
    {arr:data.temp,val:r.temperature,el:'tempValue',chart:'tempChart'},
    {arr:data.humidity,val:r.humidity,el:'humidityValue',chart:'humidityChart'},
    {arr:data.sound,val:r.sound,el:'soundValue',chart:'soundChart'},
    {arr:data.hr,val:r.heart_rate,el:'hrValue',chart:'hrChart'}
  ].forEach(s=>{
    s.arr.push({time:t,value:s.val});
    if(s.arr.length>MAX_POINTS) s.arr.shift();
    document.getElementById(s.el).textContent = s.val.toFixed(1);
    drawLineChart(s.chart,s.arr);
  });

  updateStressDetails(r);
  updateOverallStress(r);
  drawCorrelation('tempHrChart',data.temp,data.hr,'Temperature');
  drawCorrelation('soundHrChart',data.sound,data.hr,'Noise');
}

/* LINE CHART */
function drawLineChart(id,arr){
  const el = document.getElementById(id);
  el.innerHTML='';
  if(!arr.length) return;

  const w=el.clientWidth,h=el.clientHeight,m={t:10,r:10,b:20,l:40};
  const x=d3.scaleTime().domain(d3.extent(arr,d=>d.time)).range([m.l,w-m.r]);
  const y=d3.scaleLinear().domain(d3.extent(arr,d=>d.value)).nice().range([h-m.b,m.t]);

  const svg=d3.select(el).append('svg').attr('viewBox',`0 0 ${w} ${h}`);
  const line=d3.line().x(d=>x(d.time)).y(d=>y(d.value)).curve(d3.curveMonotoneX);

  svg.append('path')
    .datum(arr)
    .attr('fill','none')
    .attr('stroke','#00d4aa')
    .attr('stroke-width',2)
    .attr('d',line);
}

/* STRESS DETAILS */
function updateStressDetails(r){
  const el=document.getElementById('stressDetails');
  el.innerHTML='';

  const items=[
    {i:'🌡',l:'Temperature',v:`${r.temperature}°C`,s:r.temperature>28?'Elevated':'Normal',c:r.temperature>28?'high':'normal'},
    {i:'💧',l:'Humidity',v:`${r.humidity}%`,s:r.humidity>70?'High':'Normal',c:r.humidity>70?'warning':'normal'},
    {i:'🔊',l:'Noise',v:`${r.sound} dB`,s:r.sound>500?'Stressful':'Normal',c:r.sound>500?'high':'normal'},
    {i:'❤️',l:'Heart Rate',v:`${r.heart_rate} BPM`,s:r.heart_rate>100?'Elevated':'Normal',c:r.heart_rate>100?'high':'normal'}
  ];

  items.forEach(x=>{
    const d=document.createElement('div');
    d.className='stress-item';
    d.innerHTML=`<span>${x.i} ${x.l}: ${x.v}</span><span class="stress-state ${x.c}">${x.s}</span>`;
    el.appendChild(d);
  });
}

/* OVERALL STRESS */
function updateOverallStress(r){
  let score=0;
  if(r.temperature>28) score++;
  if(r.humidity>70) score++;
  if(r.sound>500) score++;
  if(r.heart_rate>100) score++;

  const el=document.getElementById('overallStress');
  if(score>=3){el.textContent='HIGH';el.className='stress-level high';}
  else if(score===2){el.textContent='MODERATE';el.className='stress-level moderate';}
  else{el.textContent='NORMAL';el.className='stress-level normal';}
}

/* CORRELATION */
function drawCorrelation(id,xArr,yArr,label){
  const el=document.getElementById(id);
  el.innerHTML='';
  const data=xArr.map((d,i)=>({x:d.value,y:yArr[i]?.value})).filter(d=>d.y!==undefined);
  if(!data.length) return;

  const w=el.clientWidth,h=el.clientHeight,m={t:10,r:10,b:30,l:40};
  const x=d3.scaleLinear().domain(d3.extent(data,d=>d.x)).range([m.l,w-m.r]);
  const y=d3.scaleLinear().domain(d3.extent(data,d=>d.y)).range([h-m.b,m.t]);

  const svg=d3.select(el).append('svg').attr('viewBox',`0 0 ${w} ${h}`);
  svg.selectAll('circle')
    .data(data)
    .enter()
    .append('circle')
    .attr('cx',d=>x(d.x))
    .attr('cy',d=>y(d.y))
    .attr('r',4)
    .attr('fill','#00d4aa')
    .attr('opacity',0.7);

  svg.append('text')
    .attr('x',w/2)
    .attr('y',h-5)
    .attr('fill','#94a3b8')
    .attr('font-size','10')
    .attr('text-anchor','middle')
    .text(`${label} vs Heart Rate`);
}

connect();

