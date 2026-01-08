const barWidth = 16;

export const swapOption = {
    dataset: {
        source:[['swap', 0, 8]]
    },
    grid: {
        left: 8,
        right: 8,
        bottom: 0,
        top: 0,
        containLabel: true
    },
    xAxis: {
        max: 10,
        splitLine: {
            show: false,
        },
        axisLabel: {
            show: false,
        },
        axisLine: { show: false },
        axisTick: { show: false },
        interval: 4,
    },
    yAxis: {
        data: ['swap'],
        splitLine: {
            show: false
        },
        axisLabel: {
            show: false,
        },
        axisLine: { show: false },
        axisTick: { show: false }
    },
    series: [
        {
            label: {
                show: true,
                formatter: function(params) {
                    const used = params.data[1];
                    const free = params.data[2];
                    const total = used + free;
                    return used + '/' + total + 'GB';
                }
            },
            name: 'fill',
            type: 'bar',
            barWidth: barWidth,
            itemStyle: {
                color: '#e6b600',
                borderRadius: 2,
            },
            encode:{
                x: 1,
                y: 0
            },
            z: 5,
        },

        {
            name: 'empty',
            type: 'bar',
            barWidth: barWidth,
            stack: 'total',
            barGap: '-100%',
            itemStyle: {
                opacity:.3,
                color: '#999',
                borderRadius: [2,0,0,2],
            },
            encode:{
                x: 2,
                y: 0
            },
        },
    ]
};
